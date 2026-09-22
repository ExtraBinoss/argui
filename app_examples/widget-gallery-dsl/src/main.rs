#[cfg(not(target_arch = "wasm32"))]
use argui::platform::file_picker::{FileDialog, FileFilter, FilePickerMode};
use argui::{render::RendererConfig, runtime::run_app};
#[cfg(not(feature = "argui-live"))]
use argui_example_widget_gallery_dsl::Main;

/// Launches the DSL gallery in AOT or live development mode.
///
/// # Errors
///
/// Returns connection, window, or renderer startup failures.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "argui-live")]
    let mut root = argui_dsl_runtime::LiveRuntime::connect(
        std::env::var("ARGUI_DEV_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4777".into()),
        std::time::Duration::from_secs(10),
    )?;
    #[cfg(feature = "argui-live")]
    {
        let root_id = root.root().ok_or("DSL gallery has no root component")?;
        let component_id = root
            .inspect()
            .instances
            .into_iter()
            .find(|instance| instance.id == root_id)
            .ok_or("DSL gallery root is not mounted")?
            .component;
        let callback = root
            .ir()
            .components
            .iter()
            .find(|component| component.id == component_id)
            .and_then(|component| component.callbacks.first())
            .ok_or("DSL gallery file request callback is missing")?
            .id;
        root.bind_callback(root_id, callback, |arguments| {
            let mode = match arguments.first() {
                Some(argui_dsl_runtime::DslValue::String(mode)) => mode.as_str(),
                _ => return argui_dsl_runtime::DslValue::String("Invalid file request.".into()),
            };
            argui_dsl_runtime::DslValue::String(open_native_file_dialog(mode))
        })?;
    }
    #[cfg(not(feature = "argui-live"))]
    let root = Main::new();
    #[cfg(not(feature = "argui-live"))]
    root.on_request_file(|mode| open_native_file_dialog(&mode));
    run_app(
        argui_example_widget_gallery_dsl::application_config(),
        RendererConfig::default().effects(argui_effects::registry()?),
        root,
        |_| {},
    )?;
    Ok(())
}

/// Opens the system dialog selected by `mode` and returns its outcome for the DSL status line.
///
/// `mode` is one of `file`, `files`, `folder`, `folders`, or `save`.
/// The returned string reports selected paths, dismissal, or a dialog error.
#[cfg(not(target_arch = "wasm32"))]
fn open_native_file_dialog(mode: &str) -> String {
    let dialog = match mode {
        "file" => FileDialog::new(FilePickerMode::File)
            .title("Open a document")
            .filter(FileFilter::new("Documents", ["txt", "md", "pdf"])),
        "files" => FileDialog::new(FilePickerMode::Files)
            .title("Select images")
            .filter(FileFilter::new("Images", ["png", "jpg", "jpeg", "webp"])),
        "folder" => FileDialog::new(FilePickerMode::Folder).title("Choose project folder"),
        "folders" => FileDialog::new(FilePickerMode::Folders).title("Choose source folders"),
        "save" => FileDialog::new(FilePickerMode::Save)
            .title("Choose export destination")
            .file_name("report.txt"),
        _ => return "Unknown file request.".into(),
    };
    match pollster::block_on(dialog.open()) {
        Ok(Some(files)) if !files.is_empty() => {
            let paths = files
                .iter()
                .map(|file| file.path().to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            format!("Selected {}:\n{}", paths.len(), paths.join("\n"))
        }
        Ok(_) => "No new selection.".into(),
        Err(error) => format!("Dialog failed: {error}"),
    }
}

/// Returns a platform status when a synchronous native picker cannot run in the browser.
///
/// `mode` is the requested file operation; the return value explains availability.
#[cfg(target_arch = "wasm32")]
fn open_native_file_dialog(_mode: &str) -> String {
    "Use the native gallery for system file dialogs.".into()
}
