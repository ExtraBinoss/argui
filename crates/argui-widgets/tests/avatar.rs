use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_paint::{CornerRadii, ImageId};
use argui_text::TextEngine;
use argui_ui::{ElementKind, Role, UiTree};
use argui_widgets::{Avatar, shadcn};

#[test]
fn loaded_images_and_fallbacks_share_size_clipping_and_accessible_name() {
    let palette = shadcn(Color::WHITE);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = palette.resolve(scheme);
        for image in [None, Some(ImageId::fresh())] {
            let avatar = Avatar::new("ada", "Ada Lovelace", "AL")
                .image(image)
                .size(64.0)
                .build(theme);
            assert_eq!(avatar.paint.quad.radii, CornerRadii::all(32.0));
            assert!(avatar.style.overflow.x.clips() && avatar.style.overflow.y.clips());
            assert_eq!(
                matches!(avatar.children[0].kind, ElementKind::Image { .. }),
                image.is_some()
            );
            let mut tree = UiTree::new(avatar);
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(320.0, 200.0))
                .unwrap();
            assert_eq!(output.nodes[0].bounds.size, Size::new(64.0, 64.0));
            let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
            assert_eq!(semantic.nodes.len(), 1);
            assert_eq!(semantic.nodes[0].semantics.role, Role::Image);
            assert_eq!(
                semantic.nodes[0].semantics.label.as_deref(),
                Some("Ada Lovelace")
            );
        }
    }
}

#[test]
fn invalid_sizes_remain_finite_and_fallback_is_the_default() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    for size in [-1.0, 0.0, f32::NAN, f32::INFINITY] {
        let mut tree = UiTree::new(
            Avatar::new("fallback", "Guest", "?")
                .size(size)
                .build(theme),
        );
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(100.0, 100.0))
            .unwrap();
        assert!(output.nodes[0].bounds.size.width.is_finite());
        assert!(output.nodes[0].bounds.size.width >= 1.0);
    }
    assert!(matches!(
        Avatar::new("default", "Guest", "?").build(theme).children[0].kind,
        ElementKind::Text { .. }
    ));
}
