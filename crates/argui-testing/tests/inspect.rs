use argui_accessibility::{Role, SemanticState, SemanticValue, Semantics};
use argui_runtime::{Context, Render};
use argui_testing::{Selector, SelectorCount, SemanticMatcher, TestApp, TestError};
use argui_ui::{Display, Element, length};
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

struct EdgeInspector;

impl Render for EdgeInspector {
    /// Renders semantic values, hidden keys, and duplicate visible keys.
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::column([
            Element::container([])
                .keyed("value")
                .semantics(
                    Semantics::new(Role::TextInput)
                        .value(SemanticValue::Text("semantic value".to_owned())),
                )
                .width(length(40.0))
                .height(length(20.0)),
            Element::text("hidden")
                .keyed("hidden")
                .display(Display::None),
            Element::container([])
                .keyed("zero")
                .width(length(0.0))
                .height(length(0.0)),
            Element::text("duplicate").keyed("duplicate"),
            Element::text("duplicate").keyed("duplicate"),
        ])
    }
}

/// Queries distinguish semantic values, hidden keys, empty bounds, and ambiguity.
#[test]
fn semantic_value_and_visibility_edges_have_actionable_diagnostics() {
    let mut app = TestApp::new(EdgeInspector);
    app.assert_text("semantic value");
    app.assert_exists(Selector::text("semantic value"));
    assert!(matches!(
        app.bounds("hidden"),
        Err(TestError::Selector {
            count: SelectorCount::None,
            ..
        })
    ));
    assert!(matches!(
        app.bounds("duplicate"),
        Err(TestError::Selector {
            count: SelectorCount::Multiple(2),
            ..
        })
    ));
    assert!(matches!(
        app.focused().focus(),
        Err(TestError::Selector {
            count: SelectorCount::None,
            ..
        })
    ));
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            app.assert_visible("zero");
        }))
        .is_err()
    );
}
