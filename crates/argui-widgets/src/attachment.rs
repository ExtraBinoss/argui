use crate::{Item, WidgetTheme};
use argui_ui::{Element, SemanticState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AttachmentState {
    Idle,
    Uploading,
    Processing,
    Error,
    #[default]
    Done,
}

/// File presentation only. Transfers, cancellation and opening files belong to the consumer.
#[derive(Clone, Debug)]
pub struct Attachment {
    pub item: Item,
    pub state: AttachmentState,
    pub progress: Option<f32>,
}

impl Attachment {
    /// Creates an attachment presentation for `filename`.
    /// `key` identifies this attachment; `filename` is the displayed file name.
    #[must_use]
    pub fn new(key: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            item: Item::new(key, filename),
            state: AttachmentState::Done,
            progress: None,
        }
    }

    #[must_use]
    /// Builds the attachment element using `theme` for styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let state_label = match self.state {
            AttachmentState::Idle => "Ready",
            AttachmentState::Uploading => "Uploading",
            AttachmentState::Processing => "Processing",
            AttachmentState::Error => "Transfer failed",
            AttachmentState::Done => "Uploaded",
        };
        let status = match self.progress.filter(|value| value.is_finite()) {
            Some(progress) => format!("{state_label} · {:.0}%", progress.clamp(0.0, 1.0) * 100.0),
            None => state_label.to_owned(),
        };
        let mut item = self.item;
        item.description = Some(item.description.map_or(status.clone(), |description| {
            format!("{description} · {status}")
        }));
        let mut root = item.build(theme);
        root.semantics.as_mut().expect("item semantics").state = SemanticState {
            busy: matches!(
                self.state,
                AttachmentState::Uploading | AttachmentState::Processing
            ),
            ..Default::default()
        };
        if self.state == AttachmentState::Error {
            root = root.border(argui_paint::Border::all(1.0, theme.destructive));
        }
        root
    }
}
