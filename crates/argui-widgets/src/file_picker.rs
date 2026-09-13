use argui_platform::file_picker::{
    FileDialog, FileDialogBackend, FileDialogError, FileDialogResult, NativeFileDialog, PickedFile,
};
use argui_runtime::{
    Context, EventEmitter, Render,
    tasks::{TaskError, TaskSlot},
};
use argui_ui::{Element, EventType, LiveRegion, Role, Semantics, UiEventKind};
use std::sync::Arc;

use crate::{Button, WidgetTheme, shadcn};

#[derive(Clone, Debug)]
pub enum FilePickerEvent {
    Selected(Vec<PickedFile>),
    /// No new selection was returned. The previous selection is retained.
    Dismissed,
    Failed(FileDialogError),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FilePickerStatus {
    #[default]
    Idle,
    Opening,
    Selected,
    Dismissed,
    Failed(String),
}

/// A retained component: `cx.entity(&picker)` installs the native dialog action.
/// Subscribe to `FilePickerEvent` to import files or write a chosen save destination.
pub struct FilePicker {
    key: String,
    label: String,
    pub dialog: FileDialog,
    pub enabled: bool,
    backend: Arc<dyn FileDialogBackend>,
    selection: Vec<PickedFile>,
    status: FilePickerStatus,
    task: TaskSlot,
}
impl EventEmitter<FilePickerEvent> for FilePicker {}

impl FilePicker {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, dialog: FileDialog) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            dialog,
            enabled: true,
            backend: Arc::new(NativeFileDialog),
            selection: Vec::new(),
            status: FilePickerStatus::Idle,
            task: TaskSlot::default(),
        }
    }
    /// Use an application's dialog provider with the same request/result contract.
    #[must_use]
    pub fn backend(mut self, backend: Arc<dyn FileDialogBackend>) -> Self {
        self.backend = backend;
        self
    }
    #[must_use]
    pub fn selection(&self) -> &[PickedFile] {
        &self.selection
    }
    #[must_use]
    pub fn status(&self) -> &FilePickerStatus {
        &self.status
    }
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.task.is_running()
    }

    /// Also usable from a command or menu. A second activation cannot open another dialog.
    pub fn open(&mut self, cx: &mut Context<Self>) -> Result<bool, TaskError> {
        if !self.enabled || self.is_open() {
            return Ok(false);
        }
        if let Err(error) = self.dialog.validate() {
            self.complete(Err(error), cx);
            return Ok(false);
        }
        let dialog = self.dialog.clone();
        let backend = self.backend.clone();
        let started = cx.spawn_latest(
            &mut self.task,
            async move { dialog.open_with(backend.as_ref()).await },
            |picker, result, cx| {
                picker.complete(
                    result.unwrap_or_else(|error| {
                        Err(FileDialogError::Unavailable(error.to_string()))
                    }),
                    cx,
                );
            },
        );
        if let Err(error) = started {
            self.complete(Err(FileDialogError::Unavailable(error.to_string())), cx);
            return Err(error);
        }
        self.status = FilePickerStatus::Opening;
        cx.notify();
        Ok(true)
    }

    fn complete(&mut self, result: FileDialogResult, cx: &mut Context<Self>) {
        let event = match result {
            Ok(Some(files)) if !files.is_empty() => {
                self.selection = files;
                self.status = FilePickerStatus::Selected;
                FilePickerEvent::Selected(self.selection.clone())
            }
            Ok(_) => {
                self.status = FilePickerStatus::Dismissed;
                FilePickerEvent::Dismissed
            }
            Err(error) => {
                self.status = FilePickerStatus::Failed(error.to_string());
                FilePickerEvent::Failed(error)
            }
        };
        // Ending a subscriber's resource scope must not discard the component's own result.
        let _ = cx.emit(event);
        cx.notify();
    }

    #[must_use]
    pub fn build(&mut self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let busy = self.is_open();
        let supported = self.dialog.mode.supported();
        let mut button = Button::new(&self.key, &self.label, theme.outline_button())
            .enabled(self.enabled && supported);
        if busy {
            button = button.loading(Element::text("…"));
        }
        let status = match &self.status {
            _ if !supported => "This selection mode requires the native application.".into(),
            FilePickerStatus::Idle => "No selection yet.".into(),
            FilePickerStatus::Opening => "Choose in the system dialog…".into(),
            FilePickerStatus::Selected => format!("{} selected", self.selection.len()),
            FilePickerStatus::Dismissed => "No new selection.".into(),
            FilePickerStatus::Failed(error) => error.clone(),
        };
        let mut semantics = Semantics::new(if matches!(self.status, FilePickerStatus::Failed(_)) {
            Role::Alert
        } else {
            Role::Status
        })
        .label(&status);
        semantics.live = LiveRegion::Polite;
        let mut children = vec![
            button.build(),
            Element::text(status)
                .keyed(format!("{}::status", self.key))
                .text_style(argui_text::TextStyle {
                    color: if matches!(self.status, FilePickerStatus::Failed(_)) {
                        theme.destructive
                    } else {
                        theme.muted_foreground
                    },
                    font_size: 13.0,
                    line_height: 18.0,
                    ..Default::default()
                })
                .semantics(semantics),
        ];
        for file in &self.selection {
            #[cfg(not(target_arch = "wasm32"))]
            let name = file.path().to_string_lossy().into_owned();
            #[cfg(target_arch = "wasm32")]
            let name = file.file_name();
            children.push(Element::text(name).text_style(argui_text::TextStyle {
                color: theme.foreground,
                font_size: 13.0,
                line_height: 18.0,
                ..Default::default()
            }));
        }
        Element::column(children)
            .gap(8.0)
            .keyed(format!("{}::root", self.key))
            .on(cx.listener(EventType::Click, |picker, event, cx| {
                if event.target_key() == Some(picker.key.as_str())
                    && matches!(event.kind, UiEventKind::Click(_))
                {
                    let _ = picker.open(cx);
                    event.stop_propagation();
                }
            }))
    }
}
impl Render for FilePicker {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        self.build(themes.resolve(cx.environment().color_scheme), cx)
    }
}
