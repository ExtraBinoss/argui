use argui_accessibility::{CheckedState, Role, SemanticState, Semantics};
use argui_runtime::{Context, Render};
use argui_testing::{Selector, SemanticMatcher, TestApp};
use argui_ui::Element;

struct SemanticMatrix;

impl Render for SemanticMatrix {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        let semantic = |key, label, state| {
            Element::container([])
                .keyed(key)
                .semantics(Semantics::new(Role::Button).label(label).state(state))
        };
        Element::column([
            semantic("enabled", "Enabled", SemanticState::default()),
            semantic(
                "disabled",
                "Disabled",
                SemanticState {
                    disabled: true,
                    ..SemanticState::default()
                },
            ),
            semantic(
                "checked",
                "Checked",
                SemanticState {
                    checked: Some(CheckedState::Checked),
                    ..SemanticState::default()
                },
            ),
            semantic(
                "selected",
                "Selected",
                SemanticState {
                    selected: true,
                    ..SemanticState::default()
                },
            ),
            semantic(
                "expanded",
                "Expanded",
                SemanticState {
                    expanded: Some(true),
                    ..SemanticState::default()
                },
            ),
            semantic(
                "busy",
                "Busy",
                SemanticState {
                    busy: true,
                    ..SemanticState::default()
                },
            ),
            semantic(
                "invalid",
                "Invalid",
                SemanticState {
                    invalid: true,
                    ..SemanticState::default()
                },
            ),
        ])
    }
}

#[test]
fn selectors_format_stably_and_semantic_matchers_cover_every_state() {
    let selectors = [
        (Selector::key("save"), "key(\"save\")".to_owned()),
        (
            Selector::role(Role::Button, "Save"),
            "role(Button, \"Save\")".to_owned(),
        ),
        (Selector::text("Saved"), "text(\"Saved\")".to_owned()),
        (Selector::label("Email"), "label(\"Email\")".to_owned()),
        (
            Selector::state(SemanticMatcher::Busy),
            "state(Busy)".to_owned(),
        ),
        (Selector::Focused, "focused()".to_owned()),
    ];
    for (selector, expected) in selectors {
        assert_eq!(selector.to_string(), expected);
    }
    assert_eq!(Selector::from("borrowed"), Selector::key("borrowed"));
    assert_eq!(
        Selector::from(String::from("owned")),
        Selector::key("owned")
    );

    let app = TestApp::new(SemanticMatrix);
    for (key, state) in [
        ("enabled", SemanticMatcher::Enabled),
        ("disabled", SemanticMatcher::Disabled),
        ("checked", SemanticMatcher::Checked),
        ("selected", SemanticMatcher::Selected),
        ("expanded", SemanticMatcher::Expanded),
        ("busy", SemanticMatcher::Busy),
        ("invalid", SemanticMatcher::Invalid),
    ] {
        app.assert_state(key, state);
    }
}
