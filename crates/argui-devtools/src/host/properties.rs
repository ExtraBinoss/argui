use argui_core::{Color, Key, KeyState};
use argui_inspect::{
    InspectNodeId, InspectorHandle, StyleLength, StyleProperty, StyleUnit, StyleValue,
};
use argui_runtime::{LayoutSnapshot, Render, ViewUpdate};
use argui_ui::{UiEvent, UiEventKind};
use argui_widgets::{ColorPickerState, RangeBehavior, RangeConfig, RangeState};

use super::DevtoolsHost;

#[derive(Default)]
pub(crate) struct PropertyEditing {
    pub color: Option<(InspectNodeId, StyleProperty, ColorPickerState)>,
    pub draft: Option<(String, String, bool)>,
    opacity: RangeState,
}

impl PropertyEditing {
    pub(crate) fn layout_changed(&mut self, layout: &LayoutSnapshot) {
        if let Some((node, property, color)) = &mut self.color {
            color.layout_changed(
                &format!("__devtools-color-{}-{}", node.0, property.label()),
                layout,
            );
        }
        self.opacity.layout_changed(layout, &opacity_behavior(1.0));
    }
}

fn opacity_behavior(value: f32) -> RangeBehavior {
    RangeBehavior::new(
        "__devtools-opacity",
        "Opacity",
        value,
        RangeConfig::new(0.0, 1.0, 0.01),
    )
}

fn property_target(key: &str) -> Option<(InspectNodeId, StyleProperty)> {
    let (node, label) = key.split_once('-')?;
    let node = InspectNodeId(node.parse().ok()?);
    let property = StyleProperty::ALL
        .into_iter()
        .find(|property| property.label() == label)?;
    Some((node, property))
}

pub(crate) fn current_value(
    inspector: &InspectorHandle,
    node: InspectNodeId,
    property: StyleProperty,
) -> Option<StyleValue> {
    inspector.property_value(node, property).or_else(|| {
        inspector
            .node(node)?
            .properties
            .into_iter()
            .find(|candidate| candidate.property == property)
            .map(|candidate| candidate.value)
    })
}

pub(crate) fn color_value(value: &StyleValue) -> Option<Color> {
    let rgba = match value {
        StyleValue::Srgba(rgba) => *rgba,
        StyleValue::Parameters(fields) => {
            let mut rgba = [0.0; 4];
            for (index, label) in ["red", "green", "blue", "alpha"].into_iter().enumerate() {
                rgba[index] = fields.iter().find(|field| field.label == label)?.value;
            }
            rgba
        }
        _ => return None,
    };
    Some(Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3]))
}

impl<A: Render> DevtoolsHost<A> {
    pub(super) fn property_input(&mut self, event: &UiEvent) -> Option<ViewUpdate> {
        let key = event.target_key()?;
        if let Some((node, property, color)) = &mut self.property_editing.color {
            let color_key = format!("__devtools-color-{}-{}", node.0, property.label());
            if self.inspector.selected() == Some(*node) && color.update(&color_key, event) {
                let mut value = current_value(&self.inspector, *node, *property)?;
                let rgba = color.color().to_srgba();
                match &mut value {
                    StyleValue::Parameters(fields) => {
                        for (label, value) in
                            ["red", "green", "blue", "alpha"].into_iter().zip(rgba)
                        {
                            if let Some(field) =
                                fields.iter_mut().find(|field| field.label == label)
                            {
                                field.value = value;
                            }
                        }
                    }
                    _ => value = StyleValue::Srgba(rgba),
                }
                if current_value(&self.inspector, *node, *property).as_ref() != Some(&value) {
                    self.inspector.set_property_value(*node, *property, value);
                }
                return Some(ViewUpdate::Rebuild);
            }
        }
        if key == "__devtools-opacity" {
            let node = self.inspector.selected()?;
            let StyleValue::Number(value) =
                current_value(&self.inspector, node, StyleProperty::Opacity)?
            else {
                return None;
            };
            if let Some(action) = self
                .property_editing
                .opacity
                .update(event, &opacity_behavior(value))
                && action.value().is_finite()
            {
                self.inspector.set_property_value(
                    node,
                    StyleProperty::Opacity,
                    StyleValue::Number(action.value()),
                );
                self.property_editing.draft = None;
                let _ = event.prevent_default();
                return Some(ViewUpdate::Rebuild);
            }
        }
        if let Some(suffix) = key.strip_prefix("__devtools-value-") {
            let (target, index) = suffix.rsplit_once('-')?;
            let (node, property) = property_target(target)?;
            if self.inspector.selected() != Some(node) {
                return Some(ViewUpdate::None);
            }
            let index = index.parse().ok()?;
            match &event.kind {
                UiEventKind::TextChanged(text) | UiEventKind::Submitted(text) => {
                    let mut style = current_value(&self.inspector, node, property)?;
                    let valid = text.parse::<f32>().ok().is_some_and(|value| {
                        value.is_finite()
                            && (property != StyleProperty::Opacity || (0.0..=1.0).contains(&value))
                            && (!matches!(property, StyleProperty::Width | StyleProperty::Height)
                                || value >= 0.0)
                            && style.set_field(index, value)
                    });
                    self.property_editing.draft = Some((key.into(), text.clone(), !valid));
                    if valid {
                        self.inspector.set_property_value(node, property, style);
                    }
                    return Some(ViewUpdate::Rebuild);
                }
                UiEventKind::Blurred => {
                    if self
                        .property_editing
                        .draft
                        .as_ref()
                        .is_some_and(|draft| draft.0 == key)
                    {
                        self.property_editing.draft = None;
                        return Some(ViewUpdate::Rebuild);
                    }
                }
                UiEventKind::KeyInput(input)
                    if input.key == Key::Escape && input.state == KeyState::Pressed =>
                {
                    self.property_editing.draft = None;
                    let _ = event.prevent_default();
                    return Some(ViewUpdate::Rebuild);
                }
                _ => {}
            }
        }
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return None;
        }
        if let Some(target) = key.strip_prefix("__devtools-swatch-") {
            let (node, property) = property_target(target)?;
            if self.inspector.selected() != Some(node) {
                return Some(ViewUpdate::None);
            }
            if self
                .property_editing
                .color
                .as_ref()
                .is_some_and(|(id, prop, _)| (*id, *prop) == (node, property))
            {
                self.property_editing.color = None;
            } else {
                let value = current_value(&self.inspector, node, property)?;
                self.property_editing.color = Some((
                    node,
                    property,
                    ColorPickerState::new(color_value(&value).unwrap_or(Color::TRANSPARENT)),
                ));
            }
            return Some(ViewUpdate::Rebuild);
        }
        if let Some(target) = key.strip_prefix("__devtools-reset-property-") {
            let (node, property) = property_target(target)?;
            self.inspector.clear_property_override(node, property);
            self.property_editing = PropertyEditing::default();
            return Some(ViewUpdate::Rebuild);
        }
        if let Some(suffix) = key.strip_prefix("__devtools-unit-") {
            let (target, unit) = suffix.rsplit_once('-')?;
            let (node, property) = property_target(target)?;
            if self.inspector.selected() != Some(node) {
                return Some(ViewUpdate::None);
            }
            let StyleValue::Length(old) = current_value(&self.inspector, node, property)? else {
                return None;
            };
            let unit = match unit {
                "auto" => StyleUnit::Auto,
                "px" => StyleUnit::Px,
                "percent" => StyleUnit::Percent,
                _ => return None,
            };
            let bounds = self.inspector.node(node)?.bounds;
            let value = if unit == old.unit {
                old.value
            } else if unit == StyleUnit::Percent {
                1.0
            } else if property == StyleProperty::Width {
                bounds.size.width
            } else {
                bounds.size.height
            };
            self.inspector.set_property_value(
                node,
                property,
                StyleValue::Length(StyleLength { unit, value }),
            );
            self.property_editing.draft = None;
            return Some(ViewUpdate::Rebuild);
        }
        if let Some(suffix) = key.strip_prefix("__devtools-overflow-") {
            let (node, choice) = suffix.split_once('-')?;
            let node = InspectNodeId(node.parse().ok()?);
            if self.inspector.selected() != Some(node)
                || !["Visible", "Hidden", "Auto", "Scroll"].contains(&choice)
            {
                return None;
            }
            self.inspector.set_property_value(
                node,
                StyleProperty::Overflow,
                StyleValue::Choice(format!("{choice} / {choice}")),
            );
            return Some(ViewUpdate::Rebuild);
        }
        None
    }
}
