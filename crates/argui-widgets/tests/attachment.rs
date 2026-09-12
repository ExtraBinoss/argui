use argui_core::{Color, ColorScheme};
use argui_ui::{Element, ElementKind};
use argui_widgets::{Attachment, AttachmentState, shadcn};

fn contains(element: &Element, text: &str) -> bool {
    matches!(&element.kind, ElementKind::Text { content, .. } if content.as_str().contains(text))
        || element.children.iter().any(|child| contains(child, text))
}

#[test]
fn file_states_are_textual_and_progress_is_bounded_without_starting_transfers() {
    for (state, status) in [
        (AttachmentState::Idle, "Ready"),
        (AttachmentState::Uploading, "Uploading"),
        (AttachmentState::Processing, "Processing"),
        (AttachmentState::Error, "Transfer failed"),
        (AttachmentState::Done, "Uploaded"),
    ] {
        for progress in [None, Some(-1.0), Some(0.64), Some(2.0), Some(f32::NAN)] {
            let mut attachment = Attachment::new("file", "report.pdf");
            attachment.state = state;
            attachment.progress = progress;
            if progress.is_some() {
                attachment.item.description = Some("PDF · 1 MB".into());
            }
            let root = attachment.build(shadcn(Color::BLACK).resolve(ColorScheme::Light));
            assert!(contains(&root, status));
            assert_eq!(
                root.semantics.as_ref().unwrap().state.busy,
                matches!(
                    state,
                    AttachmentState::Uploading | AttachmentState::Processing
                )
            );
            if progress == Some(2.0) {
                assert!(contains(&root, "100%"));
            }
        }
    }
}
