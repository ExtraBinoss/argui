use super::*;
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::TreeUpdate;

#[test]
fn marker_motion_without_a_changed_formatted_value_only_repaints() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    let mut state = ColorPickerState::new(Color::srgb(0.4, 0.2, 0.8));
    let view = |state: &ColorPickerState| ColorPicker::new("color", "Accent", state).build(theme);
    let mut tree = UiTree::new(view(&state));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = engine
        .compute(&mut tree, &mut text, Size::new(300.0, 400.0))
        .unwrap();
    let layouts = engine
        .custom_stats()
        .iter()
        .map(|stats| stats.phases.layouts)
        .sum::<u64>();
    let commands = output.display_list.commands().to_vec();
    let hex = field(&state, 0);
    state.set_color(Color::srgb(0.4001, 0.2, 0.8));
    assert_eq!(field(&state, 0), hex);
    assert_eq!(tree.update(view(&state)), TreeUpdate::Paint);
    engine.repaint(&tree, &mut output);
    assert_eq!(
        engine
            .custom_stats()
            .iter()
            .map(|stats| stats.phases.layouts)
            .sum::<u64>(),
        layouts
    );
    assert_ne!(output.display_list.commands(), commands);
    let resized = engine
        .compute(&mut tree, &mut text, Size::new(420.0, 400.0))
        .unwrap();
    let has_color = |shade| {
        resized.display_list.commands().iter().any(|command| matches!(command,
        argui_paint::DisplayCommand::Quad(quad) if quad.background == Some(argui_paint::Fill::Solid(Color::from_srgb8(shade, shade, shade)))))
    };
    assert!(
        has_color(230) && has_color(170),
        "both checkerboard shades survive resizing"
    );
}
