#![cfg(all(feature = "native", not(target_arch = "wasm32")))]
use argui_updater::{InstallOutcome, install::*};
use std::{fs, path::Path};

#[test]
fn format_must_match_the_installed_package() {
    let path = std::env::current_exe().unwrap();
    let detected = NativeInstaller::detect().unwrap();
    assert!(detected.supports(Format::Executable).is_ok());
    for (destination, expected) in [
        (Destination::Executable(path.clone()), Format::Executable),
        (Destination::AppImage(path.clone()), Format::AppImage),
        (Destination::AppBundle(path), Format::AppBundle),
    ] {
        let installer = NativeInstaller::new(destination);
        for format in [
            Format::Executable,
            Format::AppImage,
            Format::AppBundle,
            Format::Nsis,
            Format::Msi,
        ] {
            let supported = (format == expected
                && (format == Format::Executable
                    || (format == Format::AppImage && cfg!(target_os = "linux"))
                    || (format == Format::AppBundle && cfg!(target_os = "macos"))))
                || (matches!(format, Format::Nsis | Format::Msi) && cfg!(target_os = "windows"));
            assert_eq!(installer.supports(format).is_ok(), supported);
        }
    }
}

#[cfg(unix)]
#[test]
fn file_replacement_preserves_permissions_and_failed_install_preserves_original() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("app");
    let package = root.path().join("download");
    fs::write(&target, b"old version").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o751)).unwrap();
    fs::write(&package, b"new version").unwrap();
    let installer = NativeInstaller::new(Destination::Executable(target.clone()));
    assert!(installer.install(Format::AppBundle, &package).is_err());
    assert!(
        installer
            .install(Format::Executable, &root.path().join("missing"))
            .is_err()
    );
    assert_eq!(fs::read(&target).unwrap(), b"old version");
    assert_eq!(
        installer.install(Format::Executable, &package).unwrap(),
        InstallOutcome::RestartRequired
    );
    assert_eq!(fs::read(&target).unwrap(), b"new version");
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o751
    );
    let link = root.path().join("link");
    symlink(&target, &link).unwrap();
    assert!(
        NativeInstaller::new(Destination::Executable(link))
            .install(Format::Executable, &package)
            .is_err()
    );
    assert!(
        NativeInstaller::new(Destination::Executable(root.path().join("absent")))
            .install(Format::Executable, &package)
            .is_err()
    );
    assert!(
        NativeInstaller::new(Destination::Executable(root.path().to_owned()))
            .install(Format::Executable, &package)
            .is_err()
    );
    #[cfg(target_os = "linux")]
    assert!(
        NativeInstaller::new(Destination::AppImage(target))
            .install(Format::AppImage, &package)
            .is_ok()
    );
}

fn archive(file: &Path, entries: &[(&str, &[u8])]) {
    let gzip = flate2::write::GzEncoder::new(
        fs::File::create(file).unwrap(),
        flate2::Compression::default(),
    );
    let mut tar = tar::Builder::new(gzip);
    for (path, data) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, path, *data).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap();
}

#[test]
fn bundle_updates_replace_resources_and_reject_invalid_archives_before_moving_original() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("Example.app");
    fs::create_dir_all(target.join("Contents/MacOS")).unwrap();
    fs::write(target.join("Contents/MacOS/app"), "old").unwrap();
    fs::write(target.join("stale-resource"), "old").unwrap();
    let package = root.path().join("package.tar.gz");
    for entries in [
        vec![("Wrong.app/Contents/MacOS/app", b"new".as_slice())],
        vec![("Example.app/Contents/readme", b"new".as_slice())],
        vec![
            ("Example.app/Contents/MacOS/app", b"new".as_slice()),
            ("extra", b"extra"),
        ],
    ] {
        archive(&package, &entries);
        assert!(replace_bundle(&package, &target).is_err());
        assert_eq!(fs::read(target.join("Contents/MacOS/app")).unwrap(), b"old");
    }
    fs::write(&package, "corrupt gzip").unwrap();
    assert!(replace_bundle(&package, &target).is_err());
    assert!(replace_bundle(&root.path().join("missing"), &target).is_err());
    assert!(replace_bundle(&package, root.path()).is_err());
    assert!(replace_bundle(&package, &root.path().join("Missing.app")).is_err());
    assert!(replace_bundle(&package, Path::new("/")).is_err());
    assert!(replace_bundle(&package, Path::new("..")).is_err());
    archive(
        &package,
        &[
            ("Example.app/Contents/MacOS/app", b"new"),
            ("Example.app/Contents/Resources/icon", b"icon"),
        ],
    );
    replace_bundle(&package, &target).unwrap();
    assert_eq!(fs::read(target.join("Contents/MacOS/app")).unwrap(), b"new");
    assert!(!target.join("stale-resource").exists());
    assert_eq!(
        fs::read(target.join("Contents/Resources/icon")).unwrap(),
        b"icon"
    );
    assert_eq!(
        fs::read_dir(root.path()).unwrap().count(),
        2,
        "staging and backup must be removed"
    );
}

#[cfg(unix)]
#[test]
fn detects_packaging_in_the_launched_process_without_mutating_the_test_environment() {
    use std::process::Command;
    if let Ok(case) = std::env::var("ARGUI_UPDATER_DETECTION_CASE") {
        let detected = NativeInstaller::detect();
        if case == "relative-appimage" {
            assert!(detected.is_err());
        } else {
            let detected = detected.unwrap();
            assert!(detected.supports(Format::Executable).is_err());
            if case == "appimage" {
                assert!(detected.supports(Format::AppImage).is_ok());
            }
        }
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let bundle_exe = root.path().join("Example.app/Contents/MacOS/app");
    fs::create_dir_all(bundle_exe.parent().unwrap()).unwrap();
    fs::copy(&executable, &bundle_exe).unwrap();
    let mut cases = vec![("bundle", bundle_exe, None)];
    if cfg!(target_os = "linux") {
        cases.push((
            "appimage",
            executable.clone(),
            Some(root.path().join("App.AppImage")),
        ));
        cases.push((
            "relative-appimage",
            executable,
            Some("relative.AppImage".into()),
        ));
    }
    for (case, binary, appimage) in cases {
        let mut command = Command::new(binary);
        command
            .args([
                "--exact",
                "detects_packaging_in_the_launched_process_without_mutating_the_test_environment",
            ])
            .env("ARGUI_UPDATER_DETECTION_CASE", case)
            .env_remove("APPIMAGE");
        if let Some(path) = appimage {
            command.env("APPIMAGE", path);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{case}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
