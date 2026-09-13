use argui_core::Color;
use argui_inspect::{InspectNodeId, PropertySnapshot, StyleProperty, StyleUnit, StyleValue};
use argui_paint::{Border, CornerRadii};
use argui_ui::{AlignItems, Element, Sides, length, percent};
use argui_widgets::{Button, Checkbox, ColorPicker, Input, RangeConfig, Slider, WidgetTheme};

use super::text;
use crate::host::{DevtoolsHost, properties::color_value};

pub(super) fn property_editor<A>(
    node: InspectNodeId,
    snapshot: &PropertySnapshot,
    tools: &DevtoolsHost<A>,
    theme: &WidgetTheme,
) -> Element {
    let property = snapshot.property;
    let enabled = tools
        .inspector
        .property_enabled(node, property)
        .unwrap_or(true);
    let value = tools
        .inspector
        .property_value(node, property)
        .unwrap_or_else(|| snapshot.value.clone());
    let modified = tools.inspector.property_enabled(node, property).is_some();
    let checkbox = Checkbox::new(
        format!("__devtools-style-{}", property.label()),
        property.label(),
        if enabled {
            argui_ui::CheckedState::Checked
        } else {
            argui_ui::CheckedState::Unchecked
        },
    )
    .build(theme)
    .grow(1.0);
    let mut rows = vec![
        Element::row([
            checkbox,
            compact_button(
                format!("__devtools-reset-property-{}-{}", node.0, property.label()),
                "Reset",
                false,
                theme,
            )
            .enabled(modified)
            .build(),
        ])
        .align_items(AlignItems::CENTER)
        .gap(6.0),
    ];
    let color = color_value(&value);
    if let Some(color) =
        color.or((property == StyleProperty::Background).then_some(Color::TRANSPARENT))
    {
        rows.push(color_editor(node, property, color, tools, theme, enabled));
    }
    if let StyleValue::Length(length) = &value {
        rows.push(
            Element::row(
                [
                    ("auto", "Auto", StyleUnit::Auto),
                    ("px", "px", StyleUnit::Px),
                    ("percent", "%", StyleUnit::Percent),
                ]
                .map(|(suffix, label, unit)| {
                    compact_button(
                        format!("__devtools-unit-{}-{}-{suffix}", node.0, property.label()),
                        label,
                        length.unit == unit,
                        theme,
                    )
                    .enabled(enabled)
                    .build()
                }),
            )
            .gap(4.0),
        );
    }
    if property == StyleProperty::Overflow {
        let current = value.summary();
        rows.push(
            Element::row(["Visible", "Hidden", "Auto", "Scroll"].map(|choice| {
                compact_button(
                    format!("__devtools-overflow-{}-{choice}", node.0),
                    choice,
                    current == format!("{choice} / {choice}"),
                    theme,
                )
                .enabled(enabled)
                .build()
            }))
            .gap(3.0),
        );
    }
    if property == StyleProperty::Opacity
        && let StyleValue::Number(value) = &value
    {
        rows.push(
            Slider::new(
                "__devtools-opacity",
                "Opacity",
                *value,
                RangeConfig::new(0.0, 1.0, 0.01),
            )
            .enabled(enabled)
            .build(theme),
        );
    }
    let fields = value.fields();
    if fields.is_empty() && property != StyleProperty::Background {
        rows.push(Element::text(value.summary()).text_style(text(11.0, theme.muted_foreground)));
    }
    for (index, field) in fields.into_iter().enumerate() {
        if matches!(value, StyleValue::Srgba(_))
            || color.is_some() && ["red", "green", "blue", "alpha"].contains(&field.label.as_str())
        {
            continue;
        }
        let key = format!("__devtools-value-{}-{}-{index}", node.0, property.label());
        let draft = tools
            .property_editing
            .draft
            .as_ref()
            .filter(|draft| draft.0 == key);
        let formatted = format!("{:.3}", field.value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned();
        let invalid = draft.is_some_and(|draft| draft.2);
        let mut style = theme.input();
        style.text.font_size = 12.0;
        style.layout.padding = Sides::length(6.0);
        let input = Input::new(
            &key,
            draft.map_or(formatted, |draft| draft.1.clone()),
            "0",
            style,
        )
        .label(format!("{} {}", property.label(), field.label))
        .invalid(invalid)
        .enabled(enabled)
        .description(if invalid {
            "Enter a finite value within this property's range"
        } else {
            "Changes apply immediately"
        })
        .build()
        .width(length(104.0));
        rows.push(
            Element::row([
                Element::text(field.label)
                    .grow(1.0)
                    .text_style(text(12.0, theme.muted_foreground)),
                input,
            ])
            .align_items(AlignItems::CENTER)
            .gap(8.0),
        );
        if invalid {
            rows.push(
                Element::text("Invalid value; last valid value preserved")
                    .text_style(text(11.0, theme.destructive)),
            );
        }
    }
    Element::column(rows)
        .gap(7.0)
        .padding(Sides::length(10.0))
        .width(percent(1.0))
        .min_width(length(0.0))
        .background(theme.background)
        .border(Border::all(
            1.0,
            if modified { theme.ring } else { theme.border },
        ))
        .radius(CornerRadii::all(8.0))
}

fn color_editor<A>(
    node: InspectNodeId,
    property: StyleProperty,
    color: Color,
    tools: &DevtoolsHost<A>,
    theme: &WidgetTheme,
    enabled: bool,
) -> Element {
    let [r, g, b, a] = color.to_srgba8();
    let swatch = Element::container([])
        .width(length(24.0))
        .height(length(24.0))
        .background(color)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(4.0));
    let open = tools
        .property_editing
        .color
        .as_ref()
        .filter(|(id, prop, _)| (*id, *prop) == (node, property));
    let mut rows = vec![
        compact_button(
            format!("__devtools-swatch-{}-{}", node.0, property.label()),
            &format!("#{r:02X}{g:02X}{b:02X}{a:02X}"),
            open.is_some(),
            theme,
        )
        .leading(swatch)
        .enabled(enabled)
        .build()
        .width(percent(1.0)),
    ];
    if let Some((_, _, state)) = open {
        rows.push(
            ColorPicker::new(
                format!("__devtools-color-{}-{}", node.0, property.label()),
                property.label(),
                state,
            )
            .build(theme),
        );
    }
    Element::column(rows)
        .gap(8.0)
        .width(percent(1.0))
        .min_width(length(0.0))
}

fn compact_button(key: String, label: &str, active: bool, theme: &WidgetTheme) -> Button {
    let mut style = if active {
        theme.secondary_button()
    } else {
        theme.ghost_button()
    };
    style.layout.padding = Sides::length(6.0);
    style.label.font_size = 11.0;
    Button::new(key, label, style)
}
