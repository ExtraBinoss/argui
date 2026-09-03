use argui_core::{Point, Rect, Size};
use argui_text::{FontFamily, TextBlock, TextColor, TextScene, TextStyle};

#[test]
fn text_builders_keep_layout_ready_style() {
    let bounds = Rect::new(Point::new(4.0, 8.0), Size::new(200.0, 60.0));
    let color = TextColor::srgba(0.1, 0.2, 0.3, 0.4);
    let block = TextBlock::new("Argui", bounds)
        .size(24.0)
        .line_height(32.0)
        .color(color)
        .family(FontFamily::Named("Inter".into()))
        .weight(700)
        .clip(Rect::new(Point::new(8.0, 12.0), Size::new(80.0, 20.0)));

    assert_eq!(block.content.as_str(), "Argui");
    assert_eq!(block.bounds, bounds);
    assert_eq!(block.style.font_size, 24.0);
    assert_eq!(block.style.line_height, 32.0);
    assert_eq!(block.style.color.to_srgba8(), [26, 51, 77, 102]);
    assert_eq!(block.style.family, FontFamily::Named("Inter".into()));
    assert_eq!(block.style.weight, 700);
    assert_eq!(block.clip.origin, Point::new(8.0, 12.0));
    assert_eq!(
        TextColor::srgb(0.1, 0.2, 0.3).to_srgba8(),
        [26, 51, 77, 255]
    );
}

#[test]
fn scenes_accept_both_builder_and_mutable_insertion() {
    let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
    let mut scene = TextScene::new().with(TextBlock::new("one", bounds));
    scene.push(TextBlock::new("two", bounds));

    assert_eq!(scene.blocks().len(), 2);
    assert_eq!(TextStyle::default().family, FontFamily::SansSerif);
    assert_eq!(FontFamily::default(), FontFamily::SansSerif);
}
