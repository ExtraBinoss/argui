use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, Color, CornerRadii, DisplayCommand, DisplayList, Fill, LayerStyle, Quad,
};

#[test]
fn display_lists_preserve_cross_primitive_order() {
    let bounds = Rect::new(Point::default(), Size::new(100.0, 50.0));
    let quad = Quad {
        bounds,
        background: Some(Fill::Solid(Color::WHITE)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    };
    let mut list = DisplayList::new();
    list.push_quad(quad.clone());
    list.push_text(3);

    assert_eq!(list.quad_count(), 1);
    assert_eq!(
        list.commands(),
        &[
            DisplayCommand::Quad(quad),
            DisplayCommand::Text {
                block: 3,
                transform: Affine2D::IDENTITY,
                clips: ClipChain::default(),
            },
        ]
    );

    list.begin_layer(LayerStyle::new(Default::default()));
    list.end_layer();
    assert_eq!(list.quad_count(), 1);

    list.clear();
    assert_eq!(list.quad_count(), 0);
    assert!(list.commands().is_empty());
}
