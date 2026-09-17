use argui_accessibility::{Role, SemanticState, Semantics};
use argui_runtime::{Context, Render};
use argui_testing::{Selector, SelectorCount, SemanticMatcher, TestApp, TestError};
use argui_ui::{Element, length};
use argui_widgets::{Button, default_theme};

struct Inspectable;

impl Render for Inspectable {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        Element::column([
            Element::text("Visible value").keyed("text"),
            Button::new("focusable", "Focusable", theme.button()).build(),
            Element::container([])
                .keyed("semantic")
                .semantics(Semantics::new(Role::Button).label("Semantic label").state(
                    SemanticState {
                        busy: true,
                        ..SemanticState::default()
                    },
                ))
                .width(length(120.0))
                .height(length(32.0)),
        ])
        .width(length(240.0))
    }
}

#[test]
fn inspection_exposes_snapshots_bounds_focus_and_diagnostics() {
    let mut app = TestApp::new(Inspectable);

    assert!(!app.semantics().nodes.is_empty());
    assert!(app.dump().contains("Semantic label"));
    app.assert_text("Visible");
    app.assert_no_text("Absent");
    app.assert_exists(Selector::role(Role::Button, "Semantic label"));
    app.assert_exists(Selector::label("Semantic label"));
    app.assert_exists(Selector::text("Visible value"));
    app.assert_visible("semantic");
    app.assert_state("semantic", SemanticMatcher::Busy);
    let bounds = app.bounds("semantic").unwrap();
    app.assert_bounds("semantic", bounds);

    app.focus("focusable").unwrap();
    app.assert_focused("focusable");
    app.focused().focus().unwrap();
    app.assert_quiescent();
    app.assert_no_pending_tasks();

    let error = app.bounds("missing").unwrap_err();
    assert!(matches!(
        error,
        TestError::Selector {
            count: SelectorCount::None,
            ..
        }
    ));
    assert!(error.to_string().contains("close candidates"));
}
