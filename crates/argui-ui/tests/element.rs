#[path = "element/direction.rs"]
mod direction;
use argui_paint::CornerRadii;
use argui_text::{TextOverflow, TextStyle};
use argui_ui::{
    Color, Element, ElementKind, GpuCanvasId, GpuCanvasSpec, ImageFit, ImageId, ImageSampling,
    Insets, StateName, StylePatch, UiTree, VectorId, property,
};

#[path = "element/overrides.rs"]
mod overrides;

#[path = "element/portal.rs"]
mod portal;

const HOVERED: StateName = StateName::new("hovered-test");

#[test]
fn gpu_canvas_spec_builds_an_opaque_leaf_and_normalizes_resolution() {
    let id = GpuCanvasId::fresh();
    let spec = GpuCanvasSpec::new(id)
        .content_revision(41)
        .sampling(ImageSampling::Nearest)
        .resolution_scale(f32::INFINITY);
    let element = Element::gpu_canvas(spec);
    let ElementKind::GpuCanvas(actual) = element.kind else {
        panic!("GPU canvas constructor must create the dedicated leaf kind");
    };
    assert_eq!(actual.canvas(), id);
    assert_eq!(actual.revision(), 41);
    assert_eq!(actual.scale(), 1.0);
    assert_eq!(actual.image_sampling(), ImageSampling::Nearest);
    assert!(element.children.is_empty());
    assert_eq!(
        GpuCanvasSpec::new(id).resolution_scale(99.0).scale(),
        GpuCanvasSpec::MAX_RESOLUTION_SCALE
    );
    assert_eq!(
        GpuCanvasSpec::new(id).resolution_scale(0.01).scale(),
        GpuCanvasSpec::MIN_RESOLUTION_SCALE
    );
}

#[test]
fn final_radius_is_inherited_by_existing_visual_states() {
    let final_radius = CornerRadii::all(19.0);
    let state_radius = CornerRadii::all(7.0);
    let element = Element::container([])
        .when(
            HOVERED,
            StylePatch::new().set(property::CornerRadii, state_radius.as_array()),
        )
        .active_state(HOVERED, true)
        .radius(final_radius);
    let tree = UiTree::new(element);

    assert_eq!(
        tree.resolved_quad(tree.node_ids()[0], tree.root()).radii,
        final_radius
    );
}

#[test]
fn state_radius_authored_after_final_radius_remains_explicit() {
    let state_radius = CornerRadii::all(7.0);
    let element = Element::container([])
        .radius(CornerRadii::all(19.0))
        .when(
            HOVERED,
            StylePatch::new().set(property::CornerRadii, state_radius.as_array()),
        )
        .active_state(HOVERED, true);
    let tree = UiTree::new(element);

    assert_eq!(
        tree.resolved_quad(tree.node_ids()[0], tree.root()).radii,
        state_radius
    );
}

#[test]
fn media_and_text_specific_builders_only_change_matching_elements() {
    let vector = Element::vector(VectorId(3))
        .vector_fit(ImageFit::Fill)
        .vector_color(Color::srgb(0.2, 0.4, 0.6));
    assert!(matches!(
        vector.kind,
        ElementKind::Vector { fit: ImageFit::Fill, color, .. }
            if color == Color::srgb(0.2, 0.4, 0.6)
    ));
    assert!(matches!(
        Element::container([]).vector_fit(ImageFit::Fill).kind,
        ElementKind::Container
    ));

    let image = Element::image(ImageId(4))
        .image_fit(ImageFit::Cover)
        .image_sampling(ImageSampling::Nearest);
    assert!(matches!(
        image.kind,
        ElementKind::Image {
            fit: ImageFit::Cover,
            sampling: ImageSampling::Nearest,
            ..
        }
    ));
    assert!(matches!(
        Element::text("plain")
            .image_fit(ImageFit::Cover)
            .image_sampling(ImageSampling::Nearest)
            .kind,
        ElementKind::Text { .. }
    ));

    let style = TextStyle {
        font_size: 27.0,
        ..TextStyle::default()
    };
    assert!(matches!(
        &Element::text("styled").text_style(style.clone()).kind,
        ElementKind::Text { style: value, .. } if *value == style
    ));
    assert!(matches!(
        Element::container([]).text_style(style).kind,
        ElementKind::Container
    ));
}

#[test]
fn safe_area_builder_pads_all_edges_and_clamps_negative_adapter_values() {
    let element = Element::container([]).safe_area(Insets::new(10.0, 20.0, -1.0, 30.0));

    assert_eq!(element.children.len(), 1);
    assert_eq!(element.style.padding.top, argui_ui::length(10.0));
    assert_eq!(element.style.padding.right, argui_ui::length(20.0));
    assert_eq!(element.style.padding.bottom, argui_ui::length(0.0));
    assert_eq!(element.style.padding.left, argui_ui::length(30.0));
}

#[test]
fn empty_safe_area_keeps_the_original_element_identity() {
    let element = Element::container([]).keyed("content");

    assert_eq!(element.clone().safe_area(Insets::ZERO), element);
    assert_eq!(
        element
            .clone()
            .safe_area(Insets::new(-1.0, f32::NAN, -2.0, -3.0)),
        element
    );
}

#[test]
fn text_overflow_applies_only_to_textual_elements() {
    assert!(matches!(
        Element::text("truncate")
            .text_overflow(TextOverflow::Ellipsis(argui_text::EllipsisPosition::End))
            .kind,
        ElementKind::Text { ref style, .. } if style.overflow == TextOverflow::Ellipsis(argui_text::EllipsisPosition::End)
    ));
    assert!(matches!(
        Element::container([])
            .text_overflow(TextOverflow::Ellipsis(argui_text::EllipsisPosition::End))
            .kind,
        ElementKind::Container
    ));
}
