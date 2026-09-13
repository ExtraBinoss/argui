#![cfg(feature = "updater")]
use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    ClickEvent, Element, ElementKind, Role, SemanticValue, UiEvent, UiEventKind, UiTree,
};
use argui_updater::{InstallOutcome, Progress, ReleaseInfo, State};
use argui_widgets::{UpdateAction, UpdateDialog, shadcn};

fn info() -> ReleaseInfo {
    ReleaseInfo {
        version: "1.2.0".into(),
        notes: "Faster startup".into(),
    }
}
fn text(element: &Element, expected: &str) -> bool {
    matches!(&element.kind, ElementKind::Text { content, .. } if content.as_str() == expected)
        || element.children.iter().any(|child| text(child, expected))
}
fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), kind)
}
fn click(key: &str) -> UiEvent {
    event(key, UiEventKind::Click(ClickEvent::accessibility()))
}

#[test]
fn all_states_present_meaningful_status_and_only_offer_applicable_actions() {
    let states = [
        (
            State::Idle,
            "Check for a new version.",
            Some(UpdateAction::Check),
        ),
        (State::Checking, "Checking for updates…", None),
        (
            State::UpToDate,
            "You're up to date.",
            Some(UpdateAction::Check),
        ),
        (
            State::Available(info()),
            "A new version is available.",
            Some(UpdateAction::Download),
        ),
        (
            State::Downloading {
                release: info(),
                progress: Progress {
                    downloaded: 24_000_000,
                    total: Some(96_000_000),
                },
            },
            "24.0 / 96.0 MB · 25%",
            Some(UpdateAction::Cancel),
        ),
        (
            State::Downloading {
                release: info(),
                progress: Progress {
                    downloaded: 24_000_000,
                    total: None,
                },
            },
            "24.0 MB downloaded",
            Some(UpdateAction::Cancel),
        ),
        (
            State::Verifying(info()),
            "Verifying the download…",
            Some(UpdateAction::Cancel),
        ),
        (
            State::Ready(info()),
            "Download verified. Ready to install.",
            Some(UpdateAction::Install),
        ),
        (State::Installing(info()), "Installing the update…", None),
        (
            State::Installed {
                release: info(),
                outcome: InstallOutcome::RestartRequired,
            },
            "Update installed. Restart the application to use it.",
            None,
        ),
        (
            State::Installed {
                release: info(),
                outcome: InstallOutcome::InstallerLaunched,
            },
            "Installer started. Save your work and close the application.",
            None,
        ),
        (
            State::Cancelled(info()),
            "Download cancelled. You can try again.",
            Some(UpdateAction::Download),
        ),
        (
            State::Failed("Offline".into()),
            "Offline",
            Some(UpdateAction::Check),
        ),
    ];
    let palette = shadcn(Color::WHITE);
    for (state, status, action) in states {
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            let dialog = UpdateDialog::new("update", &state, true, Element::text("Updates"));
            assert_eq!(dialog.action(&click("update::primary")), action);
            let root = dialog.build(palette.resolve(scheme));
            assert!(text(&root, status));
            if state.release().is_some() {
                assert!(text(&root, "Faster startup"));
            }
            let mut tree = UiTree::new(root);
            let layout = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(320.0, 600.0))
                .unwrap();
            assert!(
                layout
                    .nodes
                    .iter()
                    .all(|node| node.bounds.size.width.is_finite())
            );
            let semantics = tree.semantic_tree(&layout.semantic_bounds, 1.0);
            let status_node = semantics
                .nodes
                .iter()
                .find(|node| node.semantics.label.as_deref() == Some(status))
                .unwrap();
            assert_eq!(
                status_node.semantics.role,
                if matches!(state, State::Failed(_)) {
                    Role::Alert
                } else {
                    Role::Status
                }
            );
            if let State::Downloading { progress, .. } = state {
                let bar = semantics
                    .nodes
                    .iter()
                    .find(|node| node.semantics.role == Role::Progress)
                    .unwrap();
                assert_eq!(
                    bar.semantics.value,
                    progress.percent().map(|value| SemanticValue::Number {
                        value: value as f64,
                        minimum: Some(0.0),
                        maximum: Some(100.0),
                        step: None,
                    })
                );
            }
        }
    }
}

#[test]
fn open_close_and_cancel_are_distinct_and_hidden_dialog_cannot_start_work() {
    let state = State::Downloading {
        release: info(),
        progress: Progress::default(),
    };
    let dialog = UpdateDialog::new("update", &state, true, Element::container([]));
    assert_eq!(
        dialog.action(&click("update::primary")),
        Some(UpdateAction::Cancel)
    );
    assert_eq!(
        dialog.action(&click("update::close")),
        Some(UpdateAction::Close)
    );
    assert_eq!(
        dialog.action(&click("update::backdrop")),
        Some(UpdateAction::Close)
    );
    assert_eq!(dialog.action(&click("unrelated")), None);
    assert_eq!(
        dialog.action(&event("update::primary", UiEventKind::Focused)),
        None
    );
    let escape = event(
        "update::panel",
        UiEventKind::KeyInput(KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
    assert_eq!(dialog.action(&escape), Some(UpdateAction::Close));
    let hidden = UpdateDialog::new("update", &state, false, Element::container([]));
    assert_eq!(hidden.action(&click("update::primary")), None);
    assert_eq!(
        hidden.action(&click("update::trigger")),
        Some(UpdateAction::Open)
    );
    assert_eq!(hidden.action(&escape), None);
    let palette = shadcn(Color::WHITE);
    assert!(!text(
        &hidden.build(palette.resolve(ColorScheme::Light)),
        "Update 1.2.0"
    ));
    let state = State::Available(ReleaseInfo {
        version: "2.0.0".into(),
        notes: String::new(),
    });
    let root = UpdateDialog::new("update", &state, true, Element::container([]))
        .build(palette.resolve(ColorScheme::Light));
    assert!(text(&root, "Update 2.0.0"));
    assert!(!text(&root, "Faster startup"));
}
