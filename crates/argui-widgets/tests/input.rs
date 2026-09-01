use argui_core::{Color, ColorScheme};
use argui_ui::{Dimension, ElementKind, Overflow, Role, ScrollChaining, TextInputFilter};
use argui_widgets::{Input, InputKind, TablerIcon, TextArea, WidgetAssets, shadcn};

#[test]
fn controlled_inputs_keep_semantics_and_editor_configuration_together() {
    let themes = shadcn(Color::rgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    let input = Input::new("search", "gpu", "Search", theme.input.clone())
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

    let area = TextArea::new("notes", "a\nb", "Notes", theme.input.clone()).build();
    assert_eq!(area.style.overflow.x, Overflow::Hidden);
    assert_eq!(area.style.overflow.y, Overflow::Auto);
    assert_eq!(
        area.scroll.as_ref().unwrap().chaining,
        ScrollChaining::Contain
    );
    assert!(matches!(
        area.kind,
        ElementKind::TextEditor {
            multiline: true,
            ..
        }
    ));
}

#[test]
fn numeric_input_kinds_select_engine_level_edit_filters() {
    let themes = shadcn(Color::rgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    for (kind, expected) in [
        (InputKind::Number, TextInputFilter::Decimal),
        (InputKind::Arithmetic, TextInputFilter::Arithmetic),
    ] {
        let input = Input::new("number", "12", "Value", theme.input.clone())
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
    let themes = shadcn(Color::rgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    let assets = WidgetAssets::tabler(theme.foreground);
    let input = Input::new("search", "", "Search", theme.input.clone())
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
