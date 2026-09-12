#![cfg(all(feature = "file-picker", not(target_arch = "wasm32")))]
use argui_platform::file_picker::*;
use std::sync::{Arc, Mutex};

struct Recorder(Arc<Mutex<Option<FileDialog>>>);
impl FileDialogBackend for Recorder {
    fn open(&self, dialog: FileDialog) -> FileDialogFuture {
        *self.0.lock().unwrap() = Some(dialog);
        Box::pin(async { Ok(None) })
    }
}

#[test]
fn options_reach_the_provider_and_empty_results_are_not_fake_selections() {
    for mode in [
        FilePickerMode::File,
        FilePickerMode::Files,
        FilePickerMode::Folder,
        FilePickerMode::Folders,
        FilePickerMode::Save,
    ] {
        assert!(mode.supported());
        let dialog = FileDialog::new(mode)
            .title("Select a document")
            .directory(std::env::temp_dir())
            .file_name("notes.md")
            .filter(FileFilter::new("Documents", ["md", "tar.gz"]))
            .filter(FileFilter::new("Any file", ["*"]));
        let recorded = Arc::new(Mutex::new(None));
        assert!(
            pollster::block_on(dialog.open_with(&Recorder(recorded.clone())))
                .unwrap()
                .is_none()
        );
        let sent = recorded.lock().unwrap().take().unwrap();
        assert_eq!(sent.mode, mode);
        assert_eq!(sent.title, "Select a document");
        assert_eq!(sent.file_name.as_deref(), Some("notes.md"));
        assert_eq!(sent.directory, Some(std::env::temp_dir()));
        assert_eq!(sent.filters[0].extensions, ["md", "tar.gz"]);
        assert!(format!("{sent:?}").contains("Documents"));
    }
    let default = FileDialog::default();
    assert_eq!(default.mode, FilePickerMode::File);
    assert!(default.validate().is_ok());
}

#[test]
fn invalid_options_are_rejected_before_opening_any_native_window() {
    let mut invalid = vec![FileDialog::default().title("a\0b")];
    for name in ["", ".", "..", "folder/name", "folder\\name", "a\0b"] {
        invalid.push(FileDialog::default().file_name(name));
    }
    for filter in [
        FileFilter::new("", ["txt"]),
        FileFilter::new("a\0b", ["txt"]),
        FileFilter::new("Files", [] as [&str; 0]),
    ] {
        invalid.push(FileDialog::default().filter(filter));
    }
    for extension in [
        "", ".txt", "a b", "*.rs", "a?", "a/b", "a\\b", "a;b", "a,b", "a\0b",
    ] {
        invalid.push(FileDialog::default().filter(FileFilter::new("Files", [extension])));
    }
    invalid.push(FileDialog::default().directory("bad\0path"));
    let recorded = Arc::new(Mutex::new(None));
    for dialog in invalid {
        let error =
            pollster::block_on(dialog.clone().open_with(&Recorder(recorded.clone()))).unwrap_err();
        assert!(matches!(error, FileDialogError::InvalidOptions(_)));
        assert!(!error.to_string().is_empty());
        assert!(pollster::block_on(dialog.open()).is_err());
    }
    assert!(recorded.lock().unwrap().is_none());
    assert!(
        FileDialogError::UnsupportedMode(FilePickerMode::Folder)
            .to_string()
            .contains("Folder")
    );
    assert_eq!(
        FileDialogError::Unavailable("service missing".into()).to_string(),
        "service missing"
    );
}

#[test]
fn parent_owner_is_retained_by_the_request() {
    use winit::raw_window_handle::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
    };
    struct UnavailableParent;
    impl HasWindowHandle for UnavailableParent {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            Err(HandleError::Unavailable)
        }
    }
    impl HasDisplayHandle for UnavailableParent {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
            Err(HandleError::Unavailable)
        }
    }
    let parent = Arc::new(UnavailableParent);
    let weak = Arc::downgrade(&parent);
    let dialog = FileDialog::default().parent(parent);
    assert!(weak.upgrade().is_some());
    assert!(matches!(
        pollster::block_on(dialog.open()),
        Err(FileDialogError::Unavailable(_))
    ));
    assert!(weak.upgrade().is_none());
}
