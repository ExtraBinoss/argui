use super::{DevtoolsHost, Tab, icon_element, text};
use argui_paint::{CornerRadii, PaintStyle, QuadStyle, VectorId};
use argui_ui::{
    AlignItems, Element, JustifyContent, LayoutStyle, StylePatch, length, property, sides,
};
use argui_widgets::{Button, ButtonStyle, TabsBehavior, TabsPart, WidgetTheme};

pub(super) fn toolbar<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    let tabs = TabsBehavior::new(
        "__devtools-tabs",
        [
            ("Elements".into(), true),
            ("Profiling".into(), true),
            ("Theme".into(), true),
        ],
        match tools.tab {
            Tab::Elements => 0,
            Tab::Profiling => 1,
            Tab::Theme => 2,
        },
    );
    let tab = |index, key, label, active| {
        tabs.decorate(
            TabsPart::Trigger(index),
            small_button(key, label, active, theme),
        )
        .keyed(key)
    };
    let picker = Button::icon(
        "__devtools-picker",
        "Select element",
        icon_element(
            tools.icons.target,
            18.0,
            if tools.picking {
                theme.primary_foreground
            } else {
                theme.foreground
            },
        ),
        if tools.picking {
            theme.button()
        } else {
            theme.ghost_button()
        },
    )
    .build()
    .padding(sides(7.0, 5.0));
    let mode = argui_widgets::Select::new(
        "__devtools-dock",
        "Dock position",
        crate::presentation::dock_options(tools.detach_available),
        Some(tools.dock_mode as usize),
    )
    .presence(&tools.dock_presence)
    .highlighted(tools.dock_highlight)
    .trailing(
        icon_element(tools.icons.chevron, 14.0, theme.foreground)
            .transform(argui_core::Transform2D::IDENTITY.rotate(std::f32::consts::FRAC_PI_2)),
    )
    .build(theme)
    .width(length(if tools.dock_mode == crate::DockMode::Detached {
        168.0
    } else {
        118.0
    }));
    Element::row([
        picker,
        tabs.decorate(
            TabsPart::List,
            Element::row([
                tab(
                    0,
                    "__devtools-elements",
                    "Elements",
                    tools.tab == Tab::Elements,
                ),
                tab(
                    1,
                    "__devtools-profiling",
                    "Profiling",
                    tools.tab == Tab::Profiling,
                ),
                tab(2, "__devtools-theme", "Theme", tools.tab == Tab::Theme),
            ])
            .gap(4.0),
        ),
        Element::container([]).grow(1.0).min_width(length(0.0)),
        mode,
        Button::icon(
            "__devtools-toggle",
            "Close developer tools",
            Element::text("×").text_style(text(18.0, theme.foreground)),
            theme.ghost_button(),
        )
        .build()
        .padding(sides(8.0, 5.0)),
    ])
    .min_height(length(42.0))
    .shrink(0.0)
    .gap(4.0)
    .padding(sides(6.0, 5.0))
    .background(theme.card)
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::SPACE_BETWEEN)
    .flex_wrap(argui_ui::FlexWrap::Wrap)
}

pub(super) fn toggle_button(open: bool, theme: &WidgetTheme) -> Element {
    small_button(
        "__devtools-toggle",
        if open { "×" } else { "DevTools" },
        true,
        theme,
    )
    .inspectable(false)
}

pub(super) fn small_button(key: &str, label: &str, active: bool, theme: &WidgetTheme) -> Element {
    let base = if active { theme.primary } else { theme.muted };
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(QuadStyle::solid(base).radius(CornerRadii::all(5.0))),
            text(
                12.0,
                if active {
                    theme.primary_foreground
                } else {
                    theme.foreground
                },
            ),
        )
        .layout(LayoutStyle {
            display: argui_ui::Display::Flex,
            flex_direction: argui_ui::FlexDirection::Row,
            padding: sides(9.0, 5.0),
            flex_shrink: 0.0,
            align_items: Some(AlignItems::CENTER),
            justify_content: Some(JustifyContent::CENTER),
            ..LayoutStyle::default()
        })
        .hovered(StylePatch::new().set(
            property::BackgroundColor,
            if active { theme.primary } else { theme.border },
        ))
        .pressed(
            StylePatch::new()
                .set(property::BackgroundColor, theme.primary)
                .set(property::TextColor, theme.primary_foreground),
        )
        .focused(StylePatch::new().set(property::BorderColor, theme.ring)),
    )
    .build()
}

pub(super) fn icon_label_button(
    key: &str,
    icon: VectorId,
    label: &str,
    theme: &WidgetTheme,
) -> Element {
    Button::new(key, label, theme.ghost_button())
        .leading(icon_element(icon, 16.0, theme.foreground))
        .build()
        .padding(sides(9.0, 5.0))
}
