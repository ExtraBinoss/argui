# File picker

The `file-picker` feature exposes system dialogs independently of widgets:

```toml
argui = { version = "0.2.1", features = ["file-picker"] }
```

```rust
use argui::platform::file_picker::{FileDialog, FileFilter, FilePickerMode};

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
[RFD](https://docs.rs/rfd/0.17.2/rfd/). Supply another backend through `open_with`
or the widget's `.backend(...)`.

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

## Widget

```toml
argui = { version = "0.2.1", features = ["widget-file-picker"] }
```

`widgets-all` and `argui-widgets/all` also include it. All are disabled by
default; the dialog API alone does not load widgets or their tasks.

```rust
use argui::{
    platform::file_picker::{FileDialog, FilePickerMode},
    widgets::FilePicker,
};

// Create once in your component and retain the Entity in its state.
let picker = cx.new_entity(FilePicker::new(
    "project-folder",
    "Choose a project",
    FileDialog::new(FilePickerMode::Folder).title("Project folder"),
));
// Inside render:
let element = cx.entity(&picker);
```

The component handles clicks, keyboard activation and accessibility. It displays
actual selections and errors, prevents duplicate openings and keeps the previous
selection when no new choice is returned. Read state through `selection()`,
`status()` and `is_open()`, or open from a command with `open(cx)`.
`build(theme, cx)` accepts an explicit theme.

Subscribe to `FilePickerEvent::{Selected, Dismissed, Failed}` with
`cx.subscribe(&picker, callback)` and retain the returned `Subscription`.
Subscribers and the component must share a `ModelRuntime`.

The gallery's File picker page demonstrates all five modes. The export example
selects a destination without modifying the chosen file.

## Validation

Tests isolate the native provider and cover option forwarding, validation,
selection, unchanged dismissal, errors, duplicate activation and a missing
executor. They do not capture the desktop. Check native appearance and file
selection in the gallery on each operating system.
