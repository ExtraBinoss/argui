//! Installation policy is independent of release hosting. Implement `Installer` for
//! a package manager or store; `NativeInstaller` covers directly distributed desktop apps.
use crate::{Error, InstallOutcome, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    Executable,
    AppImage,
    AppBundle,
    Nsis,
    Msi,
}

pub trait Installer {
    /// Fail during the update check when the package cannot update this installation.
    fn supports(&self, format: Format) -> Result<()>;
    /// The path belongs to a verified package and remains valid until this call returns.
    fn install(&self, format: Format, package: &Path) -> Result<InstallOutcome>;
}

#[derive(Clone, Debug)]
pub enum Destination {
    Executable(PathBuf),
    AppImage(PathBuf),
    AppBundle(PathBuf),
}

pub struct NativeInstaller {
    destination: Destination,
}

impl NativeInstaller {
    /// Useful when the app's packaging supplies an explicit installation location.
    pub fn new(destination: Destination) -> Self {
        Self { destination }
    }

    pub fn detect() -> Result<Self> {
        let exe = std::env::current_exe().map_err(Error::backend)?;
        let destination = if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some() {
            let path = PathBuf::from(std::env::var_os("APPIMAGE").unwrap_or_default());
            if !path.is_absolute() {
                return Err(Error::backend("APPIMAGE must be an absolute path"));
            }
            Destination::AppImage(path)
        } else if let Some(bundle) = exe
            .ancestors()
            .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        {
            Destination::AppBundle(bundle.to_owned())
        } else {
            Destination::Executable(exe)
        };
        Ok(Self::new(destination))
    }
}

impl Installer for NativeInstaller {
    fn supports(&self, format: Format) -> Result<()> {
        let supported = match (&self.destination, format) {
            (Destination::Executable(_), Format::Executable) => true,
            (Destination::AppImage(_), Format::AppImage) => cfg!(target_os = "linux"),
            (Destination::AppBundle(_), Format::AppBundle) => cfg!(target_os = "macos"),
            (_, Format::Nsis | Format::Msi) => cfg!(target_os = "windows"),
            _ => false,
        };
        if supported {
            Ok(())
        } else {
            Err(Error::backend(
                "update format does not match this installation",
            ))
        }
    }

    fn install(&self, format: Format, package: &Path) -> Result<InstallOutcome> {
        self.supports(format)?;
        #[cfg(target_os = "windows")]
        if matches!(format, Format::Nsis | Format::Msi) {
            return windows_installer(format, package);
        }
        match &self.destination {
            Destination::Executable(path) | Destination::AppImage(path) => {
                replace_file(package, path)?
            }
            Destination::AppBundle(path) => replace_bundle(package, path)?,
        }
        Ok(InstallOutcome::RestartRequired)
    }
}

fn replace_file(package: &Path, destination: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        if fs::canonicalize(destination).map_err(Error::backend)?
            != fs::canonicalize(std::env::current_exe().map_err(Error::backend)?)
                .map_err(Error::backend)?
        {
            return Err(Error::backend(
                "Windows executable replacement requires the running executable",
            ));
        }
        self_replace::self_replace(package).map_err(Error::backend)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let metadata = fs::symlink_metadata(destination).map_err(Error::backend)?;
        if !metadata.is_file() {
            return Err(Error::backend(
                "installation destination must be a regular file",
            ));
        }
        let parent = destination.parent().unwrap_or(Path::new("."));
        let staged = tempfile::NamedTempFile::new_in(parent).map_err(Error::backend)?;
        fs::copy(package, staged.path()).map_err(Error::backend)?;
        fs::set_permissions(staged.path(), metadata.permissions()).map_err(Error::backend)?;
        staged.as_file().sync_all().map_err(Error::backend)?;
        staged.persist(destination).map_err(Error::backend)?;
        Ok(())
    }
}

/// Replace a signed `.app.tar.gz` on the same filesystem, retaining the old bundle
/// if the final rename fails. No elevation or shell interpolation is involved.
pub fn replace_bundle(package: &Path, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| Error::backend("bundle has no parent"))?;
    let name = destination
        .file_name()
        .ok_or_else(|| Error::backend("bundle has no name"))?;
    if destination.extension().is_none_or(|ext| ext != "app") || !destination.is_dir() {
        return Err(Error::backend(
            "destination must be an existing .app bundle",
        ));
    }
    let staged = tempfile::tempdir_in(parent).map_err(Error::backend)?;
    let archive = fs::File::open(package).map_err(Error::backend)?;
    tar::Archive::new(flate2::read::GzDecoder::new(archive))
        .unpack(staged.path())
        .map_err(Error::backend)?;
    let replacement = staged.path().join(name);
    if fs::read_dir(staged.path()).map_err(Error::backend)?.count() != 1
        || !replacement.join("Contents/MacOS").is_dir()
    {
        return Err(Error::backend(
            "archive must contain one matching .app with Contents/MacOS",
        ));
    }
    let backup = tempfile::tempdir_in(parent).map_err(Error::backend)?;
    let old = backup.path().join(name);
    fs::rename(destination, &old).map_err(Error::backend)?;
    if let Err(error) = fs::rename(&replacement, destination) {
        if let Err(restore) = fs::rename(&old, destination) {
            let retained = backup.keep();
            return Err(Error::backend(format!(
                "{error}; rollback failed: {restore}; backup: {}",
                retained.display()
            )));
        }
        return Err(Error::backend(error));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_installer(format: Format, package: &Path) -> Result<InstallOutcome> {
    use std::process::Command;
    let suffix = if format == Format::Msi {
        ".msi"
    } else {
        ".exe"
    };
    let staged = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .map_err(Error::backend)?;
    fs::copy(package, staged.path()).map_err(Error::backend)?;
    let (_, path) = staged.keep().map_err(Error::backend)?;
    let result = if format == Format::Msi {
        let root = std::env::var_os("SYSTEMROOT")
            .ok_or_else(|| Error::backend("SYSTEMROOT is missing"))?;
        Command::new(PathBuf::from(root).join("System32/msiexec.exe"))
            .arg("/i")
            .arg(&path)
            .args(["/passive", "/norestart"])
            .spawn()
    } else {
        Command::new(&path).spawn()
    };
    if let Err(error) = result {
        let _ = fs::remove_file(path);
        return Err(Error::backend(error));
    }
    // The installer owns its lifecycle. The app decides when to save and quit.
    // The temporary executable must survive until the installer finishes.
    Ok(InstallOutcome::InstallerLaunched)
}
