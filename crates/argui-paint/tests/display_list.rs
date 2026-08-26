use argui_core::{Point, Rect, Size};
use argui_paint::{Border, Color, CornerRadii, DisplayCommand, DisplayList, Quad};

#[test]
fn display_lists_preserve_cross_primitive_order() {
    let bounds = Rect::new(Point::default(), Size::new(100.0, 50.0));
    let quad = Quad {
        bounds,
        background: Color::WHITE,
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        clip: bounds,
    };
    let mut list = DisplayList::new();
    list.push_quad(quad);
    list.push_text(3);

    assert_eq!(list.quad_count(), 1);
    assert_eq!(
        list.commands(),
        &[DisplayCommand::Quad(quad), DisplayCommand::Text(3)]
    );
}
