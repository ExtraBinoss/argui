//! Asynchronous system file dialogs, usable without a widget or an Argui runtime.
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

mod native;
#[cfg(any(
    target_arch = "wasm32",
    target_os = "linux",
    target_os = "windows",
    target_os = "macos"
))]
pub use rfd::FileHandle as PickedFile;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "linux", target_os = "windows", target_os = "macos"))
))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PickedFile(PathBuf);

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "linux", target_os = "windows", target_os = "macos"))
))]
impl PickedFile {
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }

    #[must_use]
    pub fn file_name(&self) -> String {
        self.0
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "linux", target_os = "windows", target_os = "macos"))
))]
impl From<PathBuf> for PickedFile {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilePickerMode {
    #[default]
    File,
    Files,
    Folder,
    Folders,
    Save,
}
impl FilePickerMode {
    /// Browsers support opening files; desktop folders and save destinations require native APIs.
    #[must_use]
    pub const fn supported(self) -> bool {
        if cfg!(target_arch = "wasm32") {
            matches!(self, Self::File | Self::Files)
        } else {
            cfg!(any(
                target_os = "linux",
                target_os = "windows",
                target_os = "macos"
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFilter {
    pub name: String,
    /// Extensions without a leading dot, or `*` for all files.
    pub extensions: Vec<String>,
}
impl FileFilter {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogError {
    InvalidOptions(String),
    UnsupportedMode(FilePickerMode),
    Unavailable(String),
}
impl std::fmt::Display for FileDialogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOptions(message) | Self::Unavailable(message) => f.write_str(message),
            Self::UnsupportedMode(mode) => {
                write!(f, "{mode:?} dialogs are unavailable on this platform")
            }
        }
    }
}
impl std::error::Error for FileDialogError {}

/// `None` means no selection. RFD does not distinguish cancellation from an OS dialog failure.
pub type FileDialogResult = Result<Option<Vec<PickedFile>>, FileDialogError>;
#[cfg(not(target_arch = "wasm32"))]
pub type FileDialogFuture = Pin<Box<dyn Future<Output = FileDialogResult> + Send>>;
#[cfg(target_arch = "wasm32")]
pub type FileDialogFuture = Pin<Box<dyn Future<Output = FileDialogResult>>>;

/// One interface for the native service and application-supplied dialog providers.
pub trait FileDialogBackend: Send + Sync {
    fn open(&self, dialog: FileDialog) -> FileDialogFuture;
}

/// The built-in provider uses XDG portals, Windows COM dialogs and AppKit panels through RFD.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeFileDialog;

trait Parent: HasWindowHandle + HasDisplayHandle + Send + Sync {}
impl<T: HasWindowHandle + HasDisplayHandle + Send + Sync> Parent for T {}

#[derive(Clone)]
pub struct FileDialog {
    pub mode: FilePickerMode,
    pub title: String,
    pub filters: Vec<FileFilter>,
    pub directory: Option<PathBuf>,
    pub file_name: Option<String>,
    parent: Option<Arc<dyn Parent>>,
}
impl std::fmt::Debug for FileDialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileDialog")
            .field("mode", &self.mode)
            .field("title", &self.title)
            .field("filters", &self.filters)
            .field("directory", &self.directory)
            .field("file_name", &self.file_name)
            .field("parented", &self.parent.is_some())
            .finish()
    }
}
impl Default for FileDialog {
    fn default() -> Self {
        Self::new(FilePickerMode::File)
    }
}
impl FileDialog {
    #[must_use]
    pub fn new(mode: FilePickerMode) -> Self {
        Self {
            mode,
            title: String::new(),
            filters: Vec::new(),
            directory: None,
            file_name: None,
            parent: None,
        }
    }
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    #[must_use]
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }
    #[must_use]
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }
    #[must_use]
    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }
    /// Retains the owner until the native dialog finishes, including if its result future is dropped.
    #[must_use]
    pub fn parent<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>(
        mut self,
        parent: Arc<W>,
    ) -> Self {
        self.parent = Some(parent);
        self
    }
    pub fn validate(&self) -> Result<(), FileDialogError> {
        let invalid = |message: &str| FileDialogError::InvalidOptions(message.into());
        if !self.mode.supported() {
            return Err(FileDialogError::UnsupportedMode(self.mode));
        }
        if self.title.contains('\0') {
            return Err(invalid("Dialog titles cannot contain NUL"));
        }
        if let Some(name) = &self.file_name
            && (name.is_empty() || name.contains(['\0', '/', '\\']) || name == "." || name == "..")
        {
            return Err(invalid(
                "The suggested file name must be a name, not a path",
            ));
        }
        if let Some(directory) = &self.directory
            && directory.as_os_str().as_encoded_bytes().contains(&0)
        {
            return Err(invalid("The starting directory cannot contain NUL"));
        }
        for filter in &self.filters {
            if filter.name.trim().is_empty()
                || filter.name.contains('\0')
                || filter.extensions.is_empty()
            {
                return Err(invalid(
                    "File filters require a name and at least one extension",
                ));
            }
            if filter.extensions.iter().any(|extension| {
                extension != "*"
                    && (extension.is_empty()
                        || extension.starts_with('.')
                        || extension.contains(['\0', '/', '\\', '*', '?', ';', ' ', ',']))
            }) {
                return Err(invalid(
                    "Use file extensions without dots, spaces, paths or globs; * accepts every file",
                ));
            }
        }
        Ok(())
    }
    /// Call from an input handler. macOS needs its application event loop; browsers need a user gesture.
    #[must_use]
    pub fn open(self) -> FileDialogFuture {
        self.open_with(&NativeFileDialog)
    }
    #[must_use]
    pub fn open_with(self, backend: &dyn FileDialogBackend) -> FileDialogFuture {
        if let Err(error) = self.validate() {
            return Box::pin(async { Err(error) });
        }
        backend.open(self)
    }
}
