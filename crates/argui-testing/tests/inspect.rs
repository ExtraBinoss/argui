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

mod observations {
    use argui_core::{Key, Modifiers, Point, ScrollDelta};
    use argui_runtime::{Context, Render};
    use argui_testing::TestApp;
    use argui_ui::{
        Axes, Element, FocusPolicy, Interaction, Overflow, RetainedIdentity, VisualState, length,
    };

    struct ObservedSurface;

    impl Render for ObservedSurface {
        /// Displays a focusable visual surface and its current retained states.
        ///
        /// * `cx` — render context supplying the current observation snapshot.
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let identity = RetainedIdentity::new(1, 1);
            let observed = cx.observed_interaction(&identity);
            let hovered = observed.states.contains(VisualState::Hovered);
            let focused = observed.states.contains(VisualState::Focused);
            let pointer_x = observed.pointer_position.map_or(-1.0, |point| point.x);
            Element::column([
                Element::container([])
                    .keyed("surface")
                    .retained_identity(identity)
                    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
                    .width(length(100.0))
                    .height(length(60.0)),
                Element::text(format!(
                    "hovered={hovered} focused={focused} pointer_x={pointer_x}"
                )),
            ])
        }
    }

    #[test]
    fn pointer_motion_refreshes_observed_hover_and_local_position() {
        let mut app = TestApp::new(ObservedSurface);
        app.assert_text("hovered=false focused=false pointer_x=-1");

        app.pointer_move(Point::new(20.0, 20.0)).unwrap();
        app.assert_text("hovered=true focused=false pointer_x=20");

        app.pointer_move(Point::new(40.0, 20.0)).unwrap();
        app.assert_text("hovered=true focused=false pointer_x=40");

        app.pointer_move(Point::new(200.0, 20.0)).unwrap();
        app.assert_text("hovered=false focused=false pointer_x=-1");
    }

    #[test]
    fn click_and_keyboard_focus_refresh_observed_focus() {
        let mut app = TestApp::new(ObservedSurface);
        app.click("surface").unwrap();
        app.assert_text("focused=true");

        app.blur().unwrap();
        app.assert_text("focused=false");

        app.key(Key::Tab, Modifiers::default()).unwrap();
        app.assert_text("focused=true");
    }

    struct ObservedScrollSurface;

    impl Render for ObservedScrollSurface {
        /// Renders a scroll container and its retained scroll observation.
        ///
        /// * `cx` — render context supplying the current scroll snapshot.
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let identity = RetainedIdentity::new(2, 1);
            let observed = cx.observed_interaction(&identity);
            let scroll_y = observed.scroll.map_or(-1.0, |scroll| scroll.offset.y);
            let viewport_h = observed
                .scroll
                .map_or(-1.0, |scroll| scroll.viewport.height);
            let content_h = observed.scroll.map_or(-1.0, |scroll| scroll.content.height);
            Element::column([
                Element::column([Element::container([])
                    .width(length(100.0))
                    .height(length(200.0))
                    .shrink(0.0)])
                .keyed("scroll")
                .retained_identity(identity)
                .width(length(100.0))
                .height(length(50.0))
                .overflow(Axes {
                    x: Overflow::Hidden,
                    y: Overflow::Auto,
                }),
                Element::text(format!(
                    "scroll_y={scroll_y} viewport_h={viewport_h} content_h={content_h}"
                )),
            ])
        }
    }

    #[test]
    fn wheel_refreshes_retained_scroll_observation() {
        let mut app = TestApp::new(ObservedScrollSurface);
        app.assert_text("scroll_y=0 viewport_h=50 content_h=200");

        let bounds = app.bounds("scroll").unwrap();
        app.wheel(
            Point::new(bounds.origin.x + 10.0, bounds.origin.y + 10.0),
            ScrollDelta::Pixels(Point::new(0.0, -40.0)),
        )
        .unwrap();
        let offset = app.scroll_offset("scroll").unwrap().y;
        assert!(offset > 0.0);
        app.assert_text(&format!("scroll_y={offset}"));
    }
}
