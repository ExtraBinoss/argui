use argui::{
    platform::file_picker::{FileDialog, FilePickerMode},
    runtime::Context,
};

use super::AstraEditor;
use crate::workspace::Project;

impl AstraEditor {
    /// Starts the platform-appropriate project selection flow.
    pub(super) fn choose_project(&mut self, cx: &mut Context<Self>) {
        #[cfg(not(target_arch = "wasm32"))]
        self.choose_native_project(cx);
        #[cfg(target_arch = "wasm32")]
        self.choose_browser_files(cx);
    }

    /// Opens a native folder picker and scans the result away from the UI thread.
    #[cfg(not(target_arch = "wasm32"))]
    fn choose_native_project(&mut self, cx: &mut Context<Self>) {
        let dialog = FileDialog::new(FilePickerMode::Folder).title("Open a project folder");
        self.loading_project = true;
        self.notice = Some("Choose a project folder…".into());
        let started = cx.spawn_latest(
            &mut self.picker_task,
            dialog.open(),
            |editor, result, cx| match result {
                Ok(Ok(Some(files))) if !files.is_empty() => {
                    editor.scan_native_project(files[0].path().to_path_buf(), cx);
                }
                Ok(Ok(_)) => {
                    editor.loading_project = false;
                    editor.notice = Some("Folder selection cancelled".into());
                    cx.notify();
                }
                Ok(Err(error)) => {
                    editor.loading_project = false;
                    editor.notice = Some(error.to_string());
                    cx.notify();
                }
                Err(error) => {
                    editor.loading_project = false;
                    editor.notice = Some(format!("Could not open the picker: {error}"));
                    cx.notify();
                }
            },
        );
        if let Err(error) = started {
            self.loading_project = false;
            self.notice = Some(format!("Could not start the picker: {error}"));
        }
        cx.notify();
    }

    /// Runs native project discovery on Argui's blocking task executor.
    #[cfg(not(target_arch = "wasm32"))]
    fn scan_native_project(&mut self, root: std::path::PathBuf, cx: &mut Context<Self>) {
        if let Some(task) = self.scan_task.take() {
            task.cancel();
        }
        self.notice = Some(format!("Indexing {}…", root.display()));
        match cx.spawn_blocking(
            move |token| Project::from_path(root, || token.is_cancelled()),
            |editor, result, cx| match result {
                Ok(Ok(project)) => editor.install_project(project, cx),
                Ok(Err(error)) => {
                    editor.loading_project = false;
                    editor.notice = Some(error.to_string());
                    cx.notify();
                }
                Err(error) => {
                    editor.loading_project = false;
                    editor.notice = Some(format!("Project indexing failed: {error}"));
                    cx.notify();
                }
            },
        ) {
            Ok(task) => self.scan_task = Some(task),
            Err(error) => {
                self.loading_project = false;
                self.notice = Some(format!("Could not start project indexing: {error}"));
            }
        }
        cx.notify();
    }

    /// Imports selected browser files into an in-memory editable project.
    #[cfg(target_arch = "wasm32")]
    fn choose_browser_files(&mut self, cx: &mut Context<Self>) {
        let dialog = FileDialog::new(FilePickerMode::Files).title("Import project files");
        self.loading_project = true;
        self.notice = Some("Choose one or more UTF-8 project files…".into());
        let started = cx.spawn_latest(
            &mut self.picker_task,
            async move {
                let selected = dialog.open().await?;
                let Some(files) = selected else {
                    return Ok::<Option<Project>, argui::platform::file_picker::FileDialogError>(
                        None,
                    );
                };
                let mut documents = Vec::new();
                for file in files {
                    let name = file.file_name();
                    if let Ok(content) = String::from_utf8(file.read().await) {
                        documents.push((name, content));
                    }
                }
                Ok(Some(Project::from_imported_documents(
                    "browser-workspace",
                    documents,
                )))
            },
            |editor, result, cx| match result {
                Ok(Ok(Some(project))) if !project.documents.is_empty() => {
                    editor.install_project(project, cx);
                }
                Ok(Ok(_)) => {
                    editor.loading_project = false;
                    editor.notice = Some("No UTF-8 files were imported".into());
                    cx.notify();
                }
                Ok(Err(error)) => {
                    editor.loading_project = false;
                    editor.notice = Some(error.to_string());
                    cx.notify();
                }
                Err(error) => {
                    editor.loading_project = false;
                    editor.notice = Some(format!("Import failed: {error}"));
                    cx.notify();
                }
            },
        );
        if let Err(error) = started {
            self.loading_project = false;
            self.notice = Some(format!("Could not start file import: {error}"));
        }
        cx.notify();
    }

    /// Saves the active document natively or updates the browser-session checkpoint.
    pub(super) fn save_active(&mut self, cx: &mut Context<Self>) {
        #[cfg(not(target_arch = "wasm32"))]
        self.save_active_native(cx);
        #[cfg(target_arch = "wasm32")]
        {
            self.project.documents[self.active_document].mark_saved();
            self.notice = Some("Saved in this browser session".into());
            cx.notify();
        }
    }

    /// Writes the active document using Argui's blocking task executor.
    #[cfg(not(target_arch = "wasm32"))]
    fn save_active_native(&mut self, cx: &mut Context<Self>) {
        let document = self.active_document;
        let snapshot = self.project.documents[document].content.clone();
        let Some(path) = self.project.documents[document].absolute_path.clone() else {
            self.project.documents[document].mark_saved();
            self.notice = Some("Saved in the bundled workspace session".into());
            cx.notify();
            return;
        };
        if let Some(task) = self.save_task.take() {
            task.cancel();
        }
        let written = snapshot.clone();
        match cx.spawn_blocking(
            move |_| std::fs::write(&path, written).map(|_| path),
            move |editor, result, cx| {
                match result {
                    Ok(Ok(path)) => {
                        if let Some(document) = editor
                            .project
                            .documents
                            .iter_mut()
                            .find(|document| document.absolute_path.as_deref() == Some(&path))
                        {
                            document.mark_snapshot_saved(&snapshot);
                            editor.notice = Some(format!("Saved {}", path.display()));
                        }
                    }
                    Ok(Err(error)) => editor.notice = Some(format!("Save failed: {error}")),
                    Err(error) => editor.notice = Some(format!("Save task failed: {error}")),
                }
                cx.notify();
            },
        ) {
            Ok(task) => self.save_task = Some(task),
            Err(error) => self.notice = Some(format!("Could not start save: {error}")),
        }
        cx.notify();
    }

    /// Launches a system terminal at the active project root when available.
    pub(super) fn launch_terminal(&mut self, cx: &mut Context<Self>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let directory = self
                .project
                .root
                .clone()
                .or_else(|| std::env::current_dir().ok());
            self.notice = Some(match directory {
                Some(directory) => match open_terminal(&directory) {
                    Ok(()) => format!("Opened terminal at {}", directory.display()),
                    Err(error) => format!("Could not open terminal: {error}"),
                },
                None => "Could not resolve the working directory".into(),
            });
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.notice = Some("System terminals are available in the native build".into());
        }
        cx.notify();
    }
}

/// Starts the default terminal application in `directory` on supported desktop systems.
#[cfg(not(target_arch = "wasm32"))]
fn open_terminal(directory: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", "Terminal"])
            .arg(directory)
            .spawn()
            .map_err(|error| error.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "cmd", "/K"])
            .current_dir(directory)
            .spawn()
            .map_err(|error| error.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        for terminal in ["x-terminal-emulator", "kgx", "gnome-terminal", "konsole"] {
            if std::process::Command::new(terminal)
                .current_dir(directory)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }
        return Err("no supported terminal application was found".into());
    }
    #[allow(unreachable_code)]
    Err("opening a terminal is unavailable on this platform".into())
}
