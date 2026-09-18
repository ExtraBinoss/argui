use argui_animation::{Duration, curves};
use argui_core::Color;
use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::TextStyle;
use argui_ui::{AlignItems, Element, JustifyContent, VisualState, length, sides};
use argui_widgets::{
    AnimatedContainer, Button, ButtonBehavior, ButtonPart, ButtonStyle, TablerIcon, WidgetAssets,
};

/// Builds one animated launcher result with its SVG icon and keyboard hint.
pub(super) fn result_row(
    result: (&str, &str, &str, &str, TablerIcon),
    index: usize,
    selected: bool,
    theme: &argui_widgets::WidgetTheme,
    icons: &WidgetAssets,
    selected_icons: &WidgetAssets,
) -> Element {
    let (id, label, description, shortcut, icon) = result;
    let foreground = if selected {
        theme.primary_foreground
    } else {
        theme.foreground
    };
    let secondary = if selected {
        theme.primary_foreground.with_alpha(0.72)
    } else {
        theme.muted_foreground
    };
    let icon_asset = if selected { selected_icons } else { icons };
    let icon_tile = AnimatedContainer::from_element(
        Element::row([icon_asset.icon(icon, 19.0)])
            .keyed(format!("spotlight-result-icon::{label}"))
            .width(length(38.0))
            .height(length(38.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER),
    )
    .background(if selected {
        Color::WHITE.with_alpha(0.16)
    } else {
        theme.muted
    })
    .radius(CornerRadii::all(if selected { 11.0 } else { 10.0 }))
    .duration(Duration::from_millis(160))
    .curve(curves::EASE_OUT)
    .build();
    let identity = Element::row([
        icon_tile,
        Element::column([
            Element::text(label).text_style(TextStyle {
                color: foreground,
                font_size: 14.0,
                line_height: 18.0,
                weight: 620,
                ..TextStyle::default()
            }),
            Element::text(description).text_style(TextStyle {
                color: secondary,
                font_size: 11.0,
                line_height: 15.0,
                weight: 450,
                ..TextStyle::default()
            }),
        ])
        .gap(1.0),
    ])
    .gap(11.0)
    .align_items(AlignItems::CENTER);
    let row = Element::row([
        identity,
        shortcut_hint_with_colors(shortcut, None, foreground, selected),
    ])
    .keyed(format!("spotlight-result::{label}"))
    .padding(sides(10.0, 7.0))
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::SPACE_BETWEEN);
    let row = AnimatedContainer::from_element(row)
        .background(if selected {
            theme.primary
        } else {
            Color::TRANSPARENT
        })
        .radius(CornerRadii::all(if selected { 12.0 } else { 9.0 }))
        .duration(Duration::from_millis(150 + index as u64 * 22))
        .curve(curves::EASE_OUT)
        .build()
        .when(
            VisualState::Hovered,
            QuadStyle::solid(if selected { theme.primary } else { theme.muted })
                .radius(CornerRadii::all(12.0))
                .into(),
        );
    ButtonBehavior::new(format!("spotlight-result::{id}"), label).decorate(ButtonPart::Root, row)
}

/// Builds an icon-only window control with an accessible label.
pub(super) fn compact_button(
    key: &str,
    label: &str,
    icon: TablerIcon,
    theme: &argui_widgets::WidgetTheme,
    icons: &WidgetAssets,
) -> Element {
    Button::icon(key, label, icons.icon(icon, 14.0), compact_style(theme))
        .build()
        .tooltip(label)
}

/// Builds a compact keyboard hint followed by its action label.
pub(super) fn shortcut_hint(key: &str, label: &str, theme: &argui_widgets::WidgetTheme) -> Element {
    shortcut_hint_with_colors(key, Some(label), theme.muted_foreground, false)
}

/// Builds a keyboard hint using colors appropriate for its surrounding result row.
fn shortcut_hint_with_colors(
    key: &str,
    label: Option<&str>,
    color: Color,
    selected: bool,
) -> Element {
    let key = Element::text(key)
        .padding(sides(7.0, 3.0))
        .background(if selected {
            Color::WHITE.with_alpha(0.15)
        } else {
            Color::BLACK.with_alpha(0.045)
        })
        .border(Border::all(
            1.0,
            if selected {
                Color::WHITE.with_alpha(0.18)
            } else {
                Color::BLACK.with_alpha(0.065)
            },
        ))
        .radius(CornerRadii::all(6.0))
        .text_style(TextStyle {
            color,
            font_size: 10.0,
            line_height: 13.0,
            weight: 650,
            ..TextStyle::default()
        });
    let mut children = vec![key];
    if let Some(label) = label {
        children.push(Element::text(label).text_style(TextStyle {
            color,
            font_size: 10.0,
            line_height: 14.0,
            weight: 550,
            ..TextStyle::default()
        }));
    }
    Element::row(children)
        .gap(5.0)
        .align_items(AlignItems::CENTER)
}

/// Builds the restrained hover and press treatment shared by footer buttons.
pub(super) fn compact_style(theme: &argui_widgets::WidgetTheme) -> ButtonStyle {
    let mut style = ButtonStyle::new(
        PaintStyle::new(
            QuadStyle::solid(theme.card)
                .border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(7.0)),
        ),
        TextStyle {
            color: theme.foreground,
            font_size: 12.0,
            line_height: 16.0,
            weight: 600,
            ..TextStyle::default()
        },
    );
    style.layout.padding = sides(10.0, 7.0);
    style.hovered = argui_ui::StylePatch::from_quad(
        QuadStyle::solid(theme.muted).radius(CornerRadii::all(7.0)),
    );
    style.pressed = argui_ui::StylePatch::from_quad(
        QuadStyle::solid(theme.muted)
            .radius(CornerRadii::all(7.0))
            .opacity(0.76),
    );
    style
}

/// Returns `color` with its alpha channel replaced by `alpha`.
pub(super) fn with_alpha(color: Color, alpha: f32) -> Color {
    color.with_alpha(alpha)
}
