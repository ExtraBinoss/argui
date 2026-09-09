use argui_core::Transform2D;
use argui_inspect::{InspectNodeId, NodeSnapshot, PropertySnapshot, StyleProperty};
use argui_paint::{CornerRadii, PaintStyle, QuadStyle, VectorId};
use argui_text::{TextColor, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, Element, LayoutStyle, Overflow, ScrollConfig, ScrollbarPartStyle,
    ScrollbarStyle, Sides, StylePatch, length, property, sides,
};
use argui_widgets::{Button, ButtonStyle, Checkbox, Input, InputStyle, WidgetTheme};

use crate::host::DevtoolsHost;

pub(crate) fn sidebar<A>(
    selected: Option<InspectNodeId>,
    nodes: &[NodeSnapshot],
    tools: &DevtoolsHost<A>,
    theme: &WidgetTheme,
) -> Element {
    let Some(node) = selected.and_then(|id| nodes.iter().find(|node| node.id == id)) else {
        return Element::text("Select an element to inspect its styles")
            .width(if tools.panel_width() < 640.0 {
                argui_ui::percent(1.0)
            } else {
                length(340.0)
            })
            .padding(Sides::length(16.0))
            .background(theme.card)
            .text_style(text(13.0, theme.muted_foreground));
    };
    let mut rows = vec![
        Element::text(format!("{}  {:?}", node.kind, node.bounds))
            .text_style(text(13.0, theme.foreground)),
        button("__devtools-reset", "Reset overrides", false, theme),
    ];
    if let Some(portal) = &node.portal {
        rows.insert(
            1,
            Element::text(format!(
                "Portal {} · {} → {} · available {:?}{}",
                portal.layer,
                portal.requested.as_deref().unwrap_or("viewport"),
                portal.resolved.as_deref().unwrap_or("viewport"),
                portal.available_size,
                if portal.constrained_width || portal.constrained_height {
                    " · constrained"
                } else {
                    ""
                }
            ))
            .text_style(text(11.0, theme.muted_foreground)),
        );
    }
    let groups = [
        ("Layout", &[StyleProperty::Width, StyleProperty::Height][..]),
        (
            "Appearance",
            &[
                StyleProperty::Background,
                StyleProperty::Border,
                StyleProperty::Opacity,
                StyleProperty::Overflow,
            ][..],
        ),
        (
            "Transforms & effects",
            &[
                StyleProperty::Transform,
                StyleProperty::Layer,
                StyleProperty::Effects,
            ][..],
        ),
    ];
    for (section, (label, properties)) in groups.into_iter().enumerate() {
        rows.push(section_header(
            section,
            label,
            tools.sections[section],
            tools.section_progress[section],
            tools.icons.chevron,
            theme,
        ));
        if tools.sections[section] {
            for property in properties {
                if let Some(snapshot) = node
                    .properties
                    .iter()
                    .find(|snapshot| snapshot.property == *property)
                {
                    rows.push(property_editor(node.id, snapshot, tools, theme));
                }
            }
        }
    }
    Element::column(rows)
        .width(length(340.0))
        .padding(Sides::length(12.0))
        .gap(7.0)
        .background(theme.card)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar(theme)))
}

fn property_editor<A>(
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
    let mut rows = vec![
        Checkbox::new(
            format!("__devtools-style-{}", property.label()),
            property.label(),
            if enabled {
                argui_ui::CheckedState::Checked
            } else {
                argui_ui::CheckedState::Unchecked
            },
        )
        .build(theme),
    ];
    let fields = value.fields();
    if fields.is_empty() {
        rows.push(
            Element::text(value.summary())
                .padding(sides(8.0, 4.0))
                .text_style(text(11.0, theme.muted_foreground)),
        );
    } else {
        rows.extend(fields.into_iter().enumerate().map(|(index, field)| {
            Element::row([
                Element::text(field.label)
                    .grow(1.0)
                    .text_style(text(11.0, theme.muted_foreground)),
                number_input(
                    &format!("__devtools-value-{}-{}-{index}", node.0, property.label()),
                    field.value,
                    theme,
                ),
            ])
            .align_items(AlignItems::CENTER)
            .gap(6.0)
            .padding(sides(8.0, 2.0))
        }));
    }
    Element::column(rows)
        .gap(3.0)
        .background(theme.background)
        .radius(CornerRadii::all(5.0))
}

fn number_input(key: &str, value: f32, theme: &WidgetTheme) -> Element {
    Input::new(
        key,
        format_number(value),
        "0",
        InputStyle::new(
            PaintStyle::new(QuadStyle::solid(theme.card).radius(CornerRadii::all(4.0))),
            text(11.0, theme.foreground),
        )
        .focused(StylePatch::new().set(property::BackgroundColor, theme.muted)),
    )
    .build()
    .width(length(92.0))
    .padding(sides(7.0, 4.0))
}

fn format_number(value: f32) -> String {
    let value = format!("{value:.3}");
    value.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn section_header(
    index: usize,
    label: &str,
    open: bool,
    progress: f32,
    icon: VectorId,
    theme: &WidgetTheme,
) -> Element {
    Button::new(
        format!("__devtools-section-{index}"),
        label,
        theme.ghost_button(),
    )
    .content(
        Element::row([
            Element::vector(icon)
                .width(length(18.0))
                .height(length(18.0))
                .shrink(0.0)
                .transform(Transform2D::IDENTITY.rotate(progress * std::f32::consts::FRAC_PI_2)),
            Element::text(label).text_style(text(12.0, theme.foreground)),
        ])
        .gap(6.0),
    )
    .build()
    .semantics(
        argui_ui::Semantics::new(argui_ui::Role::Button)
            .label(label)
            .state(argui_ui::SemanticState {
                expanded: Some(open),
                ..Default::default()
            })
            .action(argui_ui::SemanticAction::Click)
            .action(argui_ui::SemanticAction::Focus),
    )
    .height(length(29.0))
    .align_items(AlignItems::CENTER)
    .gap(6.0)
    .padding(sides(5.0, 4.0))
    .background(if open { theme.muted } else { theme.background })
}

fn button(key: &str, label: &str, active: bool, theme: &WidgetTheme) -> Element {
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
            padding: sides(10.0, 6.0),
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        })
        .hovered(StylePatch::new().set(property::BackgroundColor, theme.muted))
        .pressed(StylePatch::new().set(property::BackgroundColor, theme.primary)),
    )
    .build()
}

fn text(size: f32, color: TextColor) -> TextStyle {
    TextStyle {
        font_size: size,
        color,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    }
}

fn scrollbar(theme: &WidgetTheme) -> ScrollbarStyle {
    ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(theme.muted)),
        ScrollbarPartStyle::new(QuadStyle::solid(theme.primary).radius(CornerRadii::all(4.0))),
    )
    .width(8.0)
    .insets(argui_ui::Sides::length(3.0))
}
