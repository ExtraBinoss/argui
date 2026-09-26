use argui_animation::{Duration, Transition, Tween};
use argui_core::{Affine2D, Point, Rect, Size, Transform2D};
use argui_paint::ClipChain;
use argui_ui::{
    CursorIcon, Element, GestureSet, HitRegion, HitShape, Interaction, StylePatch, StyleTransition,
    UiTree, VisualState, property,
};
use web_time::Instant;

/// Reproducible CPU probe for a click among many unrelated styled siblings.
/// Run explicitly with nextest's ignored-test and output flags; timings are not assertions.
#[test]
#[ignore]
fn transition_click_cost() {
    for count in [32, 128, 512] {
        let nodes = (0..count).map(|_| {
            Element::container([])
                .interaction(Interaction::default())
                .when(
                    VisualState::Pressed,
                    StylePatch::new().set(
                        property::Transform,
                        Transform2D::IDENTITY.translate(0.0, 1.0),
                    ),
                )
                .transition(StyleTransition::new(Transition::tween(Tween::new(
                    Duration::from_millis(150),
                ))))
        });
        let mut tree = UiTree::new(Element::column(nodes));
        let hit = HitRegion {
            node: tree.node_ids()[1],
            bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: HitShape::Bounds,
            slop: Default::default(),
            enabled: true,
            focus_policy: Default::default(),
            cursor: CursorIcon::Auto,
            gestures: GestureSet::EMPTY,
            window_drag: None,
        };
        tree.pointer_moved(Point::new(10.0, 10.0), std::slice::from_ref(&hit));
        let start = Instant::now();
        for _ in 0..100 {
            tree.primary_pressed(std::slice::from_ref(&hit));
            tree.primary_released();
        }
        eprintln!("{count}: {:?} per click pair", start.elapsed() / 100);
    }
}
