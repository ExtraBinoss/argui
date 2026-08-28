use argui_inspect::{InspectNodeId, NodeSnapshot, PropertySnapshot, StyleProperty};
use argui_paint::{Color, CornerRadii, PaintStyle, QuadStyle, VectorId};
use argui_text::{TextColor, TextStyle, TextWrap};
use argui_ui::{
    Align, Button, ButtonStyle, Edges, Element, Interaction, LayoutStyle, Length, ScrollConfig,
    ScrollbarStyle, TextInput, TextInputStyle,
};

use crate::host::DevtoolsHost;

const BG: Color = Color::rgb(0.055, 0.065, 0.085);
const PANEL: Color = Color::rgb(0.075, 0.09, 0.12);
const TEXT: TextColor = TextColor::rgb(0.85, 0.89, 0.95);
const MUTED: TextColor = TextColor::rgb(0.55, 0.62, 0.72);
const ACCENT: Color = Color::rgb(0.25, 0.72, 0.96);

pub(crate) fn sidebar<A>(
    selected: Option<InspectNodeId>,
    nodes: &[NodeSnapshot],
    tools: &DevtoolsHost<A>,
) -> Element {
    let Some(node) = selected.and_then(|id| nodes.iter().find(|node| node.id == id)) else {
        return Element::text("Select an element to inspect its styles")
            .width(Length::Px(340.0))
            .padding(Edges::all(16.0))
            .background(PANEL)
            .text_style(text(13.0, MUTED));
    };
    let mut rows = vec![
        Element::text(format!("{}  {:?}", node.kind, node.bounds)).text_style(text(13.0, TEXT)),
        button("__devtools-reset", "Reset overrides", false),
    ];
    let groups = [
        ("Layout", &[StyleProperty::Width, StyleProperty::Height][..]),
        (
            "Appearance",
            &[
                StyleProperty::Background,
                StyleProperty::Border,
                StyleProperty::Opacity,
                StyleProperty::Clip,
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
        ));
        if tools.sections[section] {
            for property in properties {
                if let Some(snapshot) = node
                    .properties
                    .iter()
                    .find(|snapshot| snapshot.property == *property)
                {
                    rows.push(property_editor(node.id, snapshot, tools));
                }
            }
        }
    }
    Element::column(rows)
        .width(Length::Px(340.0))
        .padding(Edges::all(12.0))
        .gap(7.0)
        .background(PANEL)
        .scrollable(ScrollConfig::default().scrollbar(scrollbar()))
}

fn property_editor<A>(
    node: InspectNodeId,
    snapshot: &PropertySnapshot,
    tools: &DevtoolsHost<A>,
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
    let mut rows = vec![button(
        &format!("__devtools-style-{}", property.label()),
        &format!("{} {}", if enabled { "☑" } else { "☐" }, property.label()),
        snapshot.authored,
    )];
    let fields = value.fields();
    if fields.is_empty() {
        rows.push(
            Element::text(value.summary())
                .padding(Edges::symmetric(8.0, 4.0))
                .text_style(text(11.0, MUTED)),
        );
    } else {
        rows.extend(fields.into_iter().enumerate().map(|(index, field)| {
            Element::row([
                Element::text(field.label)
                    .grow(1.0)
                    .text_style(text(11.0, MUTED)),
                number_input(
                    &format!("__devtools-value-{}-{}-{index}", node.0, property.label()),
                    field.value,
                ),
            ])
            .align(Align::Center)
            .gap(6.0)
            .padding(Edges::symmetric(8.0, 2.0))
        }));
    }
    Element::column(rows)
        .gap(3.0)
        .background(BG)
        .radius(CornerRadii::all(5.0))
}

fn number_input(key: &str, value: f32) -> Element {
    TextInput::new(
        key,
        format_number(value),
        "0",
        TextInputStyle::new(
            PaintStyle::new(
                QuadStyle::solid(Color::rgb(0.045, 0.055, 0.075)).radius(CornerRadii::all(4.0)),
            ),
            text(11.0, TEXT),
        )
        .focused(QuadStyle::solid(Color::rgb(0.08, 0.16, 0.22))),
    )
    .build()
    .width(Length::Px(92.0))
    .padding(Edges::symmetric(7.0, 4.0))
}

fn format_number(value: f32) -> String {
    let value = format!("{value:.3}");
    value.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn section_header(index: usize, label: &str, open: bool, progress: f32, icon: VectorId) -> Element {
    Element::row([
        Element::vector(icon)
            .width(Length::Px(18.0))
            .height(Length::Px(18.0))
            .shrink(0.0)
            .vector_progress(progress),
        Element::text(label).text_style(text(12.0, TEXT)),
    ])
    .keyed(format!("__devtools-section-{index}"))
    .height(Length::Px(29.0))
    .align(Align::Center)
    .gap(6.0)
    .padding(Edges::symmetric(5.0, 4.0))
    .background(if open {
        Color::rgb(0.11, 0.15, 0.21)
    } else {
        BG
    })
    .interaction(Interaction::default().hovered(QuadStyle::solid(Color::rgb(0.15, 0.20, 0.27))))
}

fn button(key: &str, label: &str, active: bool) -> Element {
    let base = if active {
        Color::rgb(0.12, 0.32, 0.46)
    } else {
        Color::rgb(0.10, 0.12, 0.16)
    };
    Button::new(
        key,
        label,
        ButtonStyle::new(PaintStyle::new(QuadStyle::solid(base)), text(12.0, TEXT))
            .layout(LayoutStyle {
                padding: Edges::symmetric(10.0, 6.0),
                shrink: 0.0,
                ..LayoutStyle::default()
            })
            .hovered(QuadStyle::solid(Color::rgb(0.16, 0.22, 0.30)))
            .pressed(QuadStyle::solid(ACCENT)),
    )
    .build()
    .radius(CornerRadii::all(5.0))
}

fn text(size: f32, color: TextColor) -> TextStyle {
    TextStyle {
        font_size: size,
        color,
        wrap: TextWrap::None,
        ..TextStyle::default()
    }
}

fn scrollbar() -> ScrollbarStyle {
    ScrollbarStyle::new(
        QuadStyle::solid(Color::rgba(0.0, 0.0, 0.0, 0.18)),
        QuadStyle::solid(Color::rgb(0.30, 0.38, 0.48)).radius(CornerRadii::all(4.0)),
    )
    .width(8.0)
    .inset(3.0)
}
