# File picker

The `file-picker` feature exposes system dialogs independently of widgets:

```toml
argui-platform = { version = "0.3.2", features = ["file-picker"] }
```

```rust
use argui_platform::file_picker::{FileDialog, FileFilter, FilePickerMode};

let selected = FileDialog::new(FilePickerMode::Files)
    .title("Choose images")
    .filter(FileFilter::new("Images", ["png", "jpg", "webp"]))
    .open()
    .await?;

if let Some(files) = selected {
    for file in files {
        // On desktop, file.path() returns a Path without requiring UTF-8.
        println!("{}", file.file_name());
    }
}
```

Modes are `File`, `Files`, `Folder`, `Folders` and `Save`. Shared options include
title, extension filters, initial directory and suggested name. `Save` selects
a destination without creating or writing a file. Your application owns I/O
and its errors. Filters guide selection; they do not validate file contents.

`FileDialog::parent(Arc<W>)` accepts a window implementing `HasWindowHandle` and
`HasDisplayHandle`. The owner remains retained until the native dialog finishes,
even if its result is abandoned. Without a parent the dialog is standalone;
provide one when you need a modal ownership relationship.

`FileDialogBackend` is the shared interface. The default `NativeFileDialog` uses
[RFD](https://docs.rs/rfd/0.17.2/rfd/). Supply another backend through
`open_with`.

| Platform | Dialog |
| --- | --- |
| Linux Wayland/X11 | XDG Desktop Portal with the desktop's chooser; RFD can fall back to Zenity. Install a portal backend providing FileChooser and Zenity for fallback. |
| Windows | COM `IFileOpenDialog` / `IFileSaveDialog`. |
| macOS | AppKit `NSOpenPanel` / `NSSavePanel`; the application event loop must run on the main thread. |
| Web | Browser single/multiple file selection. Folder and save-destination modes return `UnsupportedMode`. |

Calls do not block Argui's event loop. In a browser, start them from a user action
to preserve browser permission to open the chooser. Dropping the future stops
receiving its result but does not necessarily close an already presented dialog.

`FileDialogResult` distinguishes a selection, no selection and configuration or
launch errors. RFD cannot distinguish user cancellation from every internal
failure: `Ok(None)` means no files were returned. NUL-containing strings,
filenames containing paths and invalid filters are rejected before native calls.

The dialog API is independent of UI components. Compose it with an application
control or workflow as needed; the caller owns file I/O and selection state.

## Validation

The platform tests cover option validation and unsupported modes. They do not
capture the desktop. Check the native chooser and file selection from an
application on each operating system.
