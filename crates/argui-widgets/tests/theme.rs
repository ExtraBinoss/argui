use argui_core::{Color, ColorScheme};
use argui_theme::ThemeMode;
use argui_ui::{Dimension, sides};
use argui_widgets::shadcn;

#[test]
fn floating_surfaces_use_scheme_specific_contrast_without_luminous_shadows() {
    let themes = shadcn(Color::BLACK);
    let light = themes.resolve(ColorScheme::Light);
    let dark = themes.resolve(ColorScheme::Dark);
    assert_eq!(light.popover, Color::WHITE);
    assert_eq!(light.popover_border, Color::BLACK.with_alpha(0.10));
    assert_eq!(dark.popover, Color::from_srgb8(24, 24, 27));
    assert_ne!(dark.popover, dark.background);
    assert_eq!(dark.popover_border, Color::WHITE.with_alpha(0.08));
    for (theme, alpha) in [(light, 0.18), (dark, 0.55)] {
        assert!(theme.foreground.contrast_ratio(theme.popover) >= 4.5);
        assert_eq!(
            theme.overlay_shadows[0].color,
            Color::BLACK.with_alpha(alpha)
        );
    }
    assert_eq!(light.dialog_backdrop, Color::BLACK.with_alpha(0.32));
    assert_eq!(dark.dialog_backdrop, Color::BLACK.with_alpha(0.62));
}

#[test]
fn all_floating_widgets_share_customizable_surface_borders_and_elevation() {
    use argui_ui::Element;
    use argui_widgets::{Dialog, Popover, Select, SelectOption, TextSelectionToolbar};
    let themes = shadcn(Color::BLACK);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let mut theme = themes.resolve(scheme).clone();
        theme.popover_border = Color::srgb(0.2, 0.4, 0.6);
        let popover = Popover::new(
            "p",
            "Popover",
            true,
            Element::text("Open"),
            Element::text("Panel"),
        )
        .build(&theme);
        let select = Select::new("s", "Select", [SelectOption::new("Item")], Some(0))
            .open(true)
            .build(&theme);
        let dialog = Dialog::new(
            "d",
            "Dialog",
            true,
            Element::text("Open"),
            Element::text("Panel"),
        )
        .build(&theme);
        let toolbar = TextSelectionToolbar::new(
            "t",
            Default::default(),
            argui_core::Size::new(800.0, 600.0),
            Default::default(),
        )
        .build(&theme);
        for panel in [
            &popover.children[1],
            &select.children[1],
            &dialog.children[1].children[1],
            &toolbar,
        ] {
            assert_eq!(
                panel.paint.quad.border,
                Some(argui_paint::Border::all(1.0, theme.popover_border))
            );
            assert_eq!(panel.layer.as_ref().unwrap().shadows, theme.overlay_shadows);
        }
    }
}

#[test]
fn floating_elevation_is_configurable_and_independent_of_blur() {
    use argui_paint::{CornerRadii, Filter, LayerMask, Shadow};

    let themes = shadcn(Color::BLACK);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let mut theme = themes.resolve(scheme).clone();
        assert!(!theme.overlay_shadows.is_empty());
        let layer = theme.overlay_layer(8.0, 12.0);
        assert_eq!(layer.shadows, theme.overlay_shadows);
        assert_eq!(layer.backdrop_filters, vec![Filter::Blur(12.0)]);
        assert_eq!(layer.mask, LayerMask::Rounded(CornerRadii::all(8.0)));
        assert!(
            layer
                .shadows
                .iter()
                .all(|shadow| !shadow.inset && shadow.blur > 0.0)
        );

        theme.overlay_shadows = vec![Shadow::drop([2.0, 3.0], 5.0, Color::BLACK)];
        let layer = theme.overlay_layer(-2.0, -1.0);
        assert_eq!(layer.shadows, theme.overlay_shadows);
        assert!(layer.backdrop_filters.is_empty());
        assert_eq!(layer.mask, LayerMask::Rounded(CornerRadii::all(0.0)));

        theme.overlay_shadows.clear();
        let layer = theme.overlay_layer(8.0, 0.0);
        assert!(layer.shadows.is_empty());
        assert!(layer.backdrop_filters.is_empty());
    }
}

#[test]
fn shadcn_palette_resolves_system_mode_and_contrasting_primary_text() {
    let mut themes = shadcn(Color::srgb(0.95, 0.95, 0.95));
    let light = themes.resolve(ColorScheme::Light).clone();
    let dark = themes.resolve(ColorScheme::Dark).clone();
    assert_ne!(light.background, dark.background);
    assert_eq!(light.primary_foreground, Color::from_srgb8(9, 9, 11));
    assert_eq!(light.background.to_srgba8(), [255, 255, 255, 255]);
    assert_eq!(light.foreground.to_srgba8(), [9, 9, 11, 255]);
    assert_eq!(dark.background.to_srgba8(), [9, 9, 11, 255]);
    assert_eq!(dark.foreground.to_srgba8(), [250, 250, 250, 255]);
    assert_eq!(dark.muted.to_srgba8(), [39, 39, 42, 255]);
    assert_eq!(dark.input_border, dark.border);
    assert!(light.foreground.contrast_ratio(light.background) >= 4.5);
    assert!(dark.foreground.contrast_ratio(dark.background) >= 4.5);
    assert_eq!(light.button().layout.size.height, Dimension::length(36.0));
    assert_eq!(light.button().layout.padding, sides(14.0, 0.0));
    themes.set_mode(ThemeMode::Dark);
    assert_eq!(themes.resolve(ColorScheme::Light), &dark);

    let dark_primary = shadcn(Color::srgb(0.05, 0.05, 0.05));
    assert_eq!(
        dark_primary.resolve(ColorScheme::Light).primary_foreground,
        Color::WHITE
    );
}

#[test]
fn explicit_light_mode_and_all_button_variants_keep_shared_dimensions() {
    let themes = shadcn(Color::WHITE).with_mode(ThemeMode::Light);
    assert_eq!(themes.mode(), ThemeMode::Light);
    let theme = themes.resolve(ColorScheme::Dark);
    assert_eq!(theme.background, Color::WHITE);
    for style in [
        theme.button(),
        theme.secondary_button(),
        theme.outline_button(),
        theme.ghost_button(),
        theme.destructive_button(),
    ] {
        assert_eq!(style.layout.size.height, Dimension::length(36.0));
        assert_eq!(style.layout.padding, sides(14.0, 0.0));
    }

    let black_palette = shadcn(Color::BLACK);
    let black_primary = black_palette.resolve(ColorScheme::Light);
    assert_eq!(black_primary.primary_foreground, Color::WHITE);
    let white_palette = shadcn(Color::WHITE);
    let white_primary = white_palette.resolve(ColorScheme::Light);
    assert_eq!(
        white_primary.primary_foreground,
        Color::from_srgb8(9, 9, 11)
    );
}

#[test]
fn control_hover_enters_and_leaves_immediately_in_both_themes() {
    use argui_core::{Point, Size};
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_ui::{CheckedState, Element, UiTree};
    use argui_widgets::{
        Button, Calendar, CalendarSelection, CalendarState, Checkbox, Date, Input, Month,
        Pagination, RadioGroup, RadioOption, Select, SelectOption, Switch, Tab, Tabs,
    };
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let date = Date::from_calendar_date(2026, Month::September, 11).unwrap();
    let calendar = CalendarState::new(date, CalendarSelection::Single(Some(date)));
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = themes.resolve(scheme);
        for (element, keys) in [
            (
                Element::row([
                    Button::new("a", "A", theme.button()).build(),
                    Button::new("b", "B", theme.ghost_button()).build(),
                ]),
                vec!["a", "b"],
            ),
            (
                Input::new("input", "Editable", "", theme.input()).build(),
                vec!["input"],
            ),
            (
                Checkbox::new("check", "Updates", CheckedState::Unchecked).build(theme),
                vec!["check"],
            ),
            (
                Switch::new("switch", "Offline", false).build(theme),
                vec!["switch"],
            ),
            (
                RadioGroup::new(
                    "radio",
                    "Quality",
                    [RadioOption::new("High"), RadioOption::new("Low")],
                    Some(0),
                )
                .build(theme),
                vec!["radio::option::0", "radio::option::1"],
            ),
            (
                Tabs::new(
                    "tabs",
                    [
                        Tab::new("A", Element::text("First")),
                        Tab::new("B", Element::text("Second")),
                    ],
                    0,
                )
                .build(theme),
                vec!["tabs::tab::1"],
            ),
            (
                Select::new(
                    "select",
                    "Choose",
                    [SelectOption::new("A"), SelectOption::new("B")],
                    Some(0),
                )
                .build(theme),
                vec!["select"],
            ),
            (
                Select::new(
                    "select",
                    "Choose",
                    [SelectOption::new("A"), SelectOption::new("B")],
                    Some(0),
                )
                .open(true)
                .build(theme),
                vec!["select::option::0", "select::option::1"],
            ),
            (
                Pagination::new("pages", 2, 12).build(theme),
                vec![
                    "pages::page::1",
                    "pages::page::2",
                    "pages::page::3",
                    "pages::page::4",
                ],
            ),
            (
                Calendar::new("calendar", "Date", &calendar, date).build(theme),
                vec![
                    "calendar::day::2026-09-09",
                    "calendar::day::2026-09-10",
                    "calendar::day::2026-09-11",
                    "calendar::day::2026-09-12",
                ],
            ),
        ] {
            let mut tree = UiTree::new(element);
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(800.0, 600.0))
                .unwrap();
            let controls: Vec<_> = keys
                .iter()
                .map(|key| {
                    let (index, &id) = tree
                        .node_ids()
                        .iter()
                        .enumerate()
                        .find(|(_, id)| tree.key(**id) == Some(key))
                        .unwrap();
                    let bounds = output
                        .hit_regions
                        .iter()
                        .find(|region| region.node == id)
                        .unwrap()
                        .bounds;
                    (
                        index,
                        id,
                        Point::new(
                            bounds.origin.x + bounds.size.width / 2.0,
                            bounds.origin.y + bounds.size.height / 2.0,
                        ),
                        tree.resolved_quad(id, tree.element_at(index).unwrap()),
                    )
                })
                .collect();
            // Cross adjacent controls without advancing the animation clock.
            for hovered in (0..controls.len()).chain((0..controls.len()).rev()) {
                tree.pointer_moved(controls[hovered].2, &output.hit_regions);
                for (index, (position, id, _, resting)) in controls.iter().enumerate() {
                    let painted = tree.resolved_quad(*id, tree.element_at(*position).unwrap());
                    if index == hovered {
                        assert_ne!(
                            &painted, resting,
                            "missing hover: {} in {scheme:?}",
                            keys[index]
                        );
                    } else {
                        assert_eq!(
                            &painted, resting,
                            "hover trail: {} in {scheme:?}",
                            keys[index]
                        );
                    }
                }
            }
            tree.pointer_moved(Point::new(-10.0, -10.0), &output.hit_regions);
            for (position, id, _, resting) in controls {
                assert_eq!(
                    tree.resolved_quad(id, tree.element_at(position).unwrap()),
                    resting
                );
            }
        }
    }
}
