use argui_ui::{Align, Direction, Edges, Element, Justify, Length, Wrap};

#[test]
fn layout_builders_expose_a_small_dsl_ready_style() {
    let element = Element::container([])
        .width(Length::Px(320.0))
        .height(Length::Percent(1.0))
        .direction(Direction::Row)
        .wrap(Wrap::Wrap)
        .align(Align::Center)
        .justify(Justify::SpaceBetween)
        .padding(Edges::symmetric(12.0, 8.0))
        .gap(6.0)
        .grow(1.0)
        .shrink(0.0);

    assert_eq!(element.style.width, Length::Px(320.0));
    assert_eq!(element.style.height, Length::Percent(1.0));
    assert_eq!(element.style.direction, Direction::Row);
    assert_eq!(element.style.wrap, Wrap::Wrap);
    assert_eq!(element.style.align, Align::Center);
    assert_eq!(element.style.justify, Justify::SpaceBetween);
    assert_eq!(element.style.padding, Edges::symmetric(12.0, 8.0));
    assert_eq!(element.style.gap, 6.0);
    assert_eq!(element.style.grow, 1.0);
    assert_eq!(element.style.shrink, 0.0);
}
