use argui_core::{Color, ColorScheme};
use argui_text::{TextContent, TextSpan, TextSpanStyle, TextWrap};
use argui_ui::{
    Dimension, ElementKind, Overflow, Role, ScrollPropagation, SemanticAction, TextInputFilter,
};
use argui_widgets::{Input, InputKind, TablerIcon, TextArea, WidgetAssets, shadcn};

#[test]
fn controlled_inputs_keep_semantics_and_editor_configuration_together() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    let input = Input::new("search", "gpu", "Search", theme.input())
        .kind(InputKind::Search)
        .label("Search components")
        .description("Command K")
        .read_only(true)
        .invalid(true)
        .build();
    let semantics = input.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, Role::SearchInput);
    assert_eq!(semantics.label.as_deref(), Some("Search components"));
    assert_eq!(semantics.description.as_deref(), Some("Command K"));
    assert!(semantics.state.read_only);
    assert!(!semantics.actions.contains(&SemanticAction::SetValue));
    assert!(semantics.state.invalid);
    assert!(matches!(
        input.kind,
        ElementKind::TextEditor {
            multiline: false,
            read_only: true,
            filter: TextInputFilter::Any,
            ..
        }
    ));

    let area = TextArea::new("notes", "a\nb", "Notes", theme.input()).build();
    assert_eq!(area.style.overflow.x, Overflow::Hidden);
    assert_eq!(area.style.overflow.y, Overflow::Auto);
    assert_eq!(
        area.scroll.as_ref().unwrap().propagation,
        ScrollPropagation::Contain
    );
    assert!(matches!(
        area.kind,
        ElementKind::TextEditor {
            multiline: true,
            ..
        }
    ));

    let code = TextArea::new("code", "fn main() {}", "Code", theme.input())
        .wrap(TextWrap::None)
        .rich_text(TextContent::rich([
            TextSpan::new("fn").style(TextSpanStyle::default().weight(700)),
            TextSpan::new(" main() {}"),
        ]))
        .build();
    assert_eq!(code.style.overflow.x, Overflow::Auto);
    assert!(matches!(
        &code.kind,
        ElementKind::TextEditor { text, styled: Some(content), .. }
            if text.wrap == TextWrap::None && content.is_rich()
    ));
}

#[test]
fn numeric_input_kinds_select_engine_level_edit_filters() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    for (kind, expected) in [
        (InputKind::Number, TextInputFilter::Decimal),
        (InputKind::Arithmetic, TextInputFilter::Arithmetic),
    ] {
        let input = Input::new("number", "12", "Value", theme.input())
            .kind(kind)
            .build();
        assert!(matches!(
            input.kind,
            ElementKind::TextEditor { filter, .. } if filter == expected
        ));
    }
}

#[test]
fn leading_content_is_overlaid_without_changing_the_editor_identity() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    let assets = WidgetAssets::tabler(theme.foreground);
    let input = Input::new("search", "", "Search", theme.input())
        .kind(InputKind::Search)
        .leading(assets.icon(TablerIcon::Search, 16.0), 38.0)
        .build();
    assert_eq!(input.children.len(), 2);
    assert!(matches!(
        input.children[0].kind,
        ElementKind::TextEditor { .. }
    ));
    assert_eq!(input.children[0].style.padding.left, argui_ui::length(38.0));
    assert_eq!(input.children[1].style.size.width, Dimension::length(38.0));
}

#[test]
fn typing_and_switching_focus_preserve_field_bounds_and_glyph_positions() {
    use argui_animation::Time;
    use argui_core::Size;
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_ui::{Element, FocusRequest, UiTree, length};

    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        for multiline in [false, true] {
            let theme = themes.resolve(scheme);
            let fields = |values: &[String; 2]| {
                Element::column(values.iter().enumerate().map(|(index, value)| {
                    let key = format!("field-{index}");
                    if multiline {
                        TextArea::new(key, value, "Notes", theme.input())
                            .build()
                            .height(length(100.0))
                    } else {
                        Input::new(key, value, "Name", theme.input()).build()
                    }
                }))
                .gap(12.0)
                .width(length(400.0))
            };
            let mut values = ["Ada Lovelace".to_owned(), String::new()];
            let mut ui = UiTree::new(fields(&values));
            let mut layout = LayoutEngine::new();
            let font = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
            let mut text = TextEngine::from_embedded_fonts(
                [font.as_slice()],
                "Noto Sans",
                "Noto Sans",
                "Noto Sans",
            );
            let viewport = Size::new(400.0, 300.0);
            let mut output = layout.compute(&mut ui, &mut text, viewport).unwrap();
            let geometry = |output: &argui_layout::LayoutOutput| {
                output
                    .text_inputs
                    .iter()
                    .map(|region| (region.bounds, region.viewport))
                    .collect::<Vec<_>>()
            };
            let resting = geometry(&output);
            let original_glyphs = text.prepare(&output.text, 1.0).glyphs;
            let original_glyphs: Vec<_> = original_glyphs
                .into_iter()
                .filter(|glyph| glyph.block == 0)
                .collect();
            assert!(!original_glyphs.is_empty());

            for (step, index) in [0, 1, 0].into_iter().enumerate() {
                let node = output.text_inputs[index].node;
                ui.sync_focus(
                    &output.hit_regions,
                    Some(FocusRequest::Focus(format!("field-{index}").into())),
                );
                ui.advance_animations(Time::from_nanos(step as u64 * 2_000_000_000 + 1));
                ui.advance_animations(Time::from_nanos(
                    step as u64 * 2_000_000_000 + 1_000_000_001,
                ));
                ui.move_text_cursor(node, values[index].len(), false);
                ui.paste_text(Some(node), " x");
                values[index] = ui.text_input_value(node).unwrap().to_owned();
                ui.update(fields(&values));
                output = layout.compute(&mut ui, &mut text, viewport).unwrap();
                assert_eq!(
                    geometry(&output),
                    resting,
                    "{scheme:?}, multiline={multiline}, field={index}"
                );
                let prepared = text.prepare(&output.text, 1.0);
                assert_eq!(&prepared.glyphs[..original_glyphs.len()], original_glyphs);
                let element = &ui.root().children[index];
                assert_eq!(
                    ui.resolved_quad(node, element).border.unwrap().color,
                    theme.primary
                );
            }
        }
    }
}
