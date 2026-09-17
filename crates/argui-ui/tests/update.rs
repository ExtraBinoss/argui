use argui_core::Transform2D;
use argui_paint::LayerStyle;
use argui_ui::{
    Color, Element, GpuCanvasId, GpuCanvasSpec, TreeUpdate, UiTree, UserSelect, length,
};

fn screen(color: Color) -> Element {
    Element::row([
        Element::column([Element::text("Volume"), Element::text("55 %")]).keyed("controls"),
        Element::container([])
            .keyed("animation")
            .width(length(80.0))
            .background(color),
    ])
}

#[test]
fn rebuilding_an_animation_retains_unchanged_controls_and_their_descendants() {
    let mut tree = UiTree::new(screen(Color::BLACK));
    let controls = tree.root().children[0].clone();
    let animation = tree.root().children[1].clone();
    let ids = tree.node_ids().to_vec();
    assert_eq!(tree.update(screen(Color::WHITE)), TreeUpdate::Paint);
    assert!(tree.root().children[0].ptr_eq(&controls));
    assert!(tree.root().children[0].children[1].ptr_eq(&controls.children[1]));
    assert!(!tree.root().children[1].ptr_eq(&animation));
    assert_eq!(tree.node_ids(), ids);
    assert_eq!(tree.update(screen(Color::WHITE)), TreeUpdate::None);
    assert!(tree.root().children[0].ptr_eq(&controls));
}

#[test]
fn retaining_equal_subtrees_does_not_hide_semantic_or_layout_edits() {
    let mut tree = UiTree::new(screen(Color::BLACK));
    let mut next = screen(Color::WHITE);
    next.children[0].semantics = Some(Box::new(
        argui_ui::Semantics::new(argui_ui::Role::Group).label("Mixer"),
    ));
    assert_eq!(tree.update(next.clone()), TreeUpdate::Paint);
    assert_eq!(
        tree.root().children[0].semantics,
        next.children[0].semantics
    );
    next.children[0].children.push(Element::text("Pan"));
    assert_eq!(tree.update(next), TreeUpdate::Layout);
    assert_eq!(tree.root().children[0].children.len(), 3);
}

#[test]
fn text_color_and_alpha_repaint_but_text_and_font_metrics_still_relayout() {
    let text = |value: &str, color, size| {
        Element::text(value).text_style(argui_text::TextStyle {
            color,
            font_size: size,
            ..Default::default()
        })
    };
    let mut tree = UiTree::new(text("Track", Color::WHITE, 14.0));
    for color in [Color::BLACK, Color::WHITE.with_alpha(0.32), Color::WHITE] {
        assert_eq!(tree.update(text("Track", color, 14.0)), TreeUpdate::Paint);
    }
    assert_eq!(
        tree.update(text("Track", Color::WHITE, 14.0)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.update(text("Track", Color::WHITE, 20.0)),
        TreeUpdate::Layout
    );
    assert_eq!(
        tree.update(text("Other", Color::WHITE, 20.0)),
        TreeUpdate::Layout
    );

    let fixed = |value| {
        Element::text(value)
            .width(length(120.0))
            .height(length(24.0))
            .user_select(UserSelect::None)
    };
    let mut fixed_tree = UiTree::new(fixed("Old"));
    fixed_tree.mark_layout_clean();
    assert_eq!(fixed_tree.update(fixed("New value")), TreeUpdate::Paint);
    assert!(!fixed_tree.layout_dirty());
    assert_eq!(
        UiTree::new(fixed("Old").height(argui_ui::Dimension::auto()))
            .update(fixed("New").height(argui_ui::Dimension::auto())),
        TreeUpdate::Layout
    );
    assert_eq!(
        UiTree::new(
            Element::text("Old")
                .width(length(120.0))
                .height(length(24.0))
        )
        .update(
            Element::text("New")
                .width(length(120.0))
                .height(length(24.0))
        ),
        TreeUpdate::Layout
    );
}

#[test]
fn gpu_canvas_revision_retains_identity_while_remount_allocates_a_new_node() {
    let canvas = GpuCanvasId::fresh();
    let view = |revision| {
        Element::gpu_canvas(GpuCanvasSpec::new(canvas).content_revision(revision)).keyed("canvas")
    };
    let mut tree = UiTree::new(Element::container([view(1)]));
    let retained = tree.node_ids()[1];
    assert_eq!(
        tree.update(Element::container([view(2)])),
        TreeUpdate::Paint
    );
    assert_eq!(tree.node_ids()[1], retained);

    assert_eq!(tree.update(Element::container([])), TreeUpdate::Layout);
    assert_eq!(
        tree.update(Element::container([view(2)])),
        TreeUpdate::Layout
    );
    assert_ne!(tree.node_ids()[1], retained);
}

#[test]
fn established_transform_and_group_opacity_changes_use_composition() {
    let moving = |x, opacity| {
        Element::container([])
            .transform(Transform2D::IDENTITY.translate(x, 0.0))
            .layer(LayerStyle::new(Default::default()).opacity(opacity))
    };
    let mut tree = UiTree::new(moving(0.0, 0.8));
    tree.mark_layout_clean();

    assert_eq!(tree.update(moving(20.0, 0.5)), TreeUpdate::Composite);
    assert!(!tree.layout_dirty());

    let plain = Element::container([]);
    assert_eq!(
        UiTree::new(plain.clone())
            .update(plain.transform(Transform2D::IDENTITY.translate(20.0, 0.0))),
        TreeUpdate::Paint
    );
}

#[test]
fn paint_changes_remain_stronger_than_compositor_changes() {
    let original = Element::container([])
        .background(Color::BLACK)
        .transform(Transform2D::IDENTITY.translate(1.0, 0.0));
    let changed = Element::container([])
        .background(Color::WHITE)
        .transform(Transform2D::IDENTITY.translate(20.0, 0.0));
    assert_eq!(UiTree::new(original).update(changed), TreeUpdate::Paint);
}
