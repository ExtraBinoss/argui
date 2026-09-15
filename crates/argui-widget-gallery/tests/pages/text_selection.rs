use argui::{
    paint::Fill,
    runtime::Entity,
    ui::{CornerRadii, UserSelect},
};
use argui_widget_gallery::WidgetGallery;

#[test]
fn selection_page_exposes_solid_rounded_gradient_and_policy_examples() {
    let gallery = Entity::new(WidgetGallery::default());
    super::click(&gallery, "nav::text-selection");
    let root = gallery.render();

    let default = super::keyed(&root, "selection-default").unwrap();
    assert!(matches!(
        default.selection_highlight.as_ref().unwrap().background,
        Fill::Solid(_)
    ));
    let rounded = super::keyed(&root, "selection-rounded").unwrap();
    assert_eq!(
        rounded.selection_highlight.as_ref().unwrap().radii,
        CornerRadii::all(7.0)
    );
    for key in ["selection-gradient", "selection-rainbow"] {
        assert!(matches!(
            super::keyed(&root, key)
                .unwrap()
                .selection_highlight
                .as_ref()
                .unwrap()
                .background,
            Fill::Linear(_)
        ));
    }
    assert_eq!(
        super::keyed(&root, "selection-policy-none")
            .unwrap()
            .user_select,
        UserSelect::None
    );

    assert!(super::contains_text(&root, "Pause rainbow"));
    super::click(&gallery, "selection-rainbow-toggle");
    assert!(super::contains_text(&gallery.render(), "Play rainbow"));
}

#[test]
fn mounted_rainbow_selection_advances_with_presentation_frames() {
    use argui::{
        animation::{Duration, Frame, Time},
        ui::{ClickEvent, UiEventKind, UiTree},
    };
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    let click = |key: &str| {
        let mut tree = UiTree::new(gallery.render(Default::default()).unwrap());
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| tree.key(*id) == Some(key))
            .unwrap();
        for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
            if event.should_dispatch() {
                gallery.dispatch_event(&event).unwrap();
            }
        }
    };
    click("nav::text-selection");
    let before_root = gallery.render(Default::default()).unwrap();
    let before = super::keyed(&before_root, "selection-rainbow")
        .unwrap()
        .selection_highlight
        .as_ref()
        .unwrap()
        .background
        .clone();

    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(40_000_000),
            elapsed: Duration::from_millis(40),
        })
        .unwrap();
    let after_root = gallery.render(Default::default()).unwrap();
    let after = &super::keyed(&after_root, "selection-rainbow")
        .unwrap()
        .selection_highlight
        .as_ref()
        .unwrap()
        .background;
    assert_ne!(&before, after);

    click("selection-rainbow-toggle");
    let paused = after.clone();
    gallery
        .animation_frame(Frame {
            now: Time::from_nanos(80_000_000),
            elapsed: Duration::from_millis(40),
        })
        .unwrap();
    let paused_root = gallery.render(Default::default()).unwrap();
    assert_eq!(
        &paused,
        &super::keyed(&paused_root, "selection-rainbow")
            .unwrap()
            .selection_highlight
            .as_ref()
            .unwrap()
            .background
    );
}
