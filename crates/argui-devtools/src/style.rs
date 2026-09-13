use argui_core::Transform2D;
use argui_inspect::{InspectNodeId, NodeSnapshot, StyleProperty};
use argui_paint::{CornerRadii, PaintStyle, QuadStyle, VectorId};
use argui_text::{TextColor, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, Element, LayoutStyle, Overflow, ScrollConfig, ScrollbarPartStyle,
    ScrollbarStyle, Sides, StylePatch, length, property, sides,
};
use argui_widgets::{Button, ButtonStyle, WidgetTheme};

use crate::host::DevtoolsHost;

mod properties;
use properties::property_editor;

pub(crate) fn sidebar<A>(
    selected: Option<InspectNodeId>,
    nodes: &[NodeSnapshot],
    tools: &DevtoolsHost<A>,
    theme: &WidgetTheme,
) -> Element {
    let Some(node) = selected.and_then(|id| nodes.iter().find(|node| node.id == id)) else {
        return Element::text("Select an element to inspect its styles")
            .width(argui_ui::percent(1.0))
            .min_width(length(0.0))
            .padding(Sides::length(16.0))
            .background(theme.card)
            .text_style(text(13.0, theme.muted_foreground));
    };
    let mut rows = vec![
        Element::text(format!(
            "{}{}",
            node.kind,
            node.key
                .as_ref()
                .map_or(String::new(), |key| format!("  #{key}"))
        ))
        .text_style(text(13.0, theme.foreground)),
        Element::text(format!(
            "{:.1} × {:.1} px · x {:.1} · y {:.1}",
            node.bounds.size.width,
            node.bounds.size.height,
            node.bounds.origin.x,
            node.bounds.origin.y
        ))
        .text_style(text(11.0, theme.muted_foreground)),
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
        let properties: Vec<_> = node
            .properties
            .iter()
            .filter(|snapshot| {
                properties.contains(&snapshot.property)
                    && (snapshot.authored
                        || tools
                            .inspector
                            .property_enabled(node.id, snapshot.property)
                            .is_some()
                        || snapshot.property == StyleProperty::Opacity
                            && (matches!(node.kind.as_str(), "image" | "vector")
                                || node.properties.iter().any(|property| {
                                    property.authored
                                        && matches!(
                                            property.property,
                                            StyleProperty::Background | StyleProperty::Border
                                        )
                                })))
            })
            .collect();
        if properties.is_empty() {
            continue;
        }
        rows.push(section_header(
            section,
            label,
            tools.sections[section],
            tools.section_progress[section],
            tools.icons.chevron,
            theme,
        ));
        if tools.sections[section] {
            for snapshot in properties {
                rows.push(property_editor(node.id, snapshot, tools, theme));
            }
        }
    }
    Element::column(rows)
        .keyed("__devtools-properties")
        .interaction(argui_ui::Interaction::blocker())
        .width(argui_ui::percent(1.0))
        .min_width(length(0.0))
        .padding(Sides::length(12.0))
        .gap(7.0)
        .background(theme.card)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar(theme)))
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
