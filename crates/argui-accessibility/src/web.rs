use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{
    Element, Event, HtmlCanvasElement, HtmlElement, HtmlInputElement, HtmlTextAreaElement,
    KeyboardEvent,
};

use crate::{
    SemanticAction, SemanticNode, SemanticNodeId, SemanticPatch, SemanticRequest, SemanticTree,
    SemanticValue, Semantics,
};

mod attributes;
use attributes::{apply_attributes, apply_bounds, html_tag};

type EventHandler = Closure<dyn FnMut(Event)>;

struct DomNode {
    element: Element,
    semantics: Semantics,
    bounds: Option<[f64; 4]>,
    _handlers: Vec<EventHandler>,
}

pub struct DomTree {
    canvas: HtmlCanvasElement,
    root: HtmlElement,
    nodes: HashMap<SemanticNodeId, DomNode>,
    snapshot: SemanticTree,
    bounds: Option<[f64; 4]>,
    on_action: std::rc::Rc<dyn Fn(SemanticRequest)>,
}

impl DomTree {
    /// Updates the native password control without storing its secret in semantic snapshots.
    pub fn set_protected_value(&self, id: SemanticNodeId, value: &str) {
        if let Some(node) = self.nodes.get(&id)
            && let Some(input) = node.element.dyn_ref::<HtmlInputElement>()
            && input.type_() == "password"
            && input.value() != value
        {
            input.set_value(value);
        }
    }
    pub fn new(
        canvas: HtmlCanvasElement,
        snapshot: SemanticTree,
        on_action: impl Fn(SemanticRequest) + 'static,
    ) -> Result<Self, JsValue> {
        let document = canvas
            .owner_document()
            .ok_or_else(|| JsValue::from_str("canvas has no owner document"))?;
        let root = document.create_element("div")?.dyn_into::<HtmlElement>()?;
        root.set_attribute("data-argui-accessibility", "")?;
        let window_key = canvas
            .get_attribute("data-argui-window")
            .unwrap_or_else(|| snapshot.root.get().to_string());
        let suffix = window_key
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        root.set_id(&format!("argui-accessibility-{suffix}"));
        canvas.set_attribute("role", "application")?;
        canvas.set_attribute("aria-owns", &root.id())?;
        let style = root.style();
        style.set_property("position", "fixed")?;
        style.set_property("z-index", "2147483647")?;
        style.set_property("pointer-events", "none")?;
        style.set_property("overflow", "hidden")?;
        let parent = canvas
            .parent_node()
            .ok_or_else(|| JsValue::from_str("canvas must be attached before accessibility"))?;
        parent.append_child(&root)?;
        let mut tree = Self {
            canvas,
            root,
            nodes: HashMap::new(),
            snapshot: snapshot.clone(),
            bounds: None,
            on_action: std::rc::Rc::new(on_action),
        };
        tree.position_root()?;
        tree.apply_full(&snapshot)?;
        Ok(tree)
    }

    pub fn sync(&mut self, next: SemanticTree) -> Result<bool, JsValue> {
        self.position_root()?;
        let patch = self.snapshot.diff(&next);
        if patch.is_empty() {
            return Ok(false);
        }
        self.apply_patch(&patch, &next)?;
        self.snapshot = next;
        Ok(true)
    }

    fn position_root(&mut self) -> Result<(), JsValue> {
        let rect = self.canvas.get_bounding_client_rect();
        apply_bounds(
            &self.root,
            &mut self.bounds,
            [rect.left(), rect.top(), rect.width(), rect.height()],
        )
    }

    fn apply_full(&mut self, tree: &SemanticTree) -> Result<(), JsValue> {
        for node in &tree.nodes {
            self.upsert(node)?;
        }
        self.attach_children(tree)?;
        self.focus(tree.focus);
        Ok(())
    }

    fn apply_patch(&mut self, patch: &SemanticPatch, tree: &SemanticTree) -> Result<(), JsValue> {
        for id in &patch.removed {
            if let Some(node) = self.nodes.remove(id) {
                node.element.remove();
            }
        }
        for node in &patch.upserts {
            self.upsert(node)?;
        }
        self.attach_children(tree)?;
        if patch.focus.is_some() {
            self.focus(tree.focus);
        }
        Ok(())
    }

    fn upsert(&mut self, node: &SemanticNode) -> Result<(), JsValue> {
        let tag = html_tag(node.semantics.role);
        let replace = self.nodes.get(&node.id).is_some_and(|current| {
            current.element.tag_name().to_ascii_lowercase() != tag
                || current.semantics.actions != node.semantics.actions
        });
        if replace && let Some(previous) = self.nodes.remove(&node.id) {
            previous.element.remove();
        }
        if !self.nodes.contains_key(&node.id) {
            let document = self
                .canvas
                .owner_document()
                .ok_or_else(|| JsValue::from_str("canvas has no owner document"))?;
            let element = document.create_element(tag)?;
            element.set_id(&format!("{}-{}", self.root.id(), node.id.get()));
            let handlers = handlers(
                node.id,
                &node.semantics.actions,
                &element,
                std::rc::Rc::clone(&self.on_action),
            )?;
            apply_attributes(&element, node, &self.root.id())?;
            let style = element.unchecked_ref::<HtmlElement>().style();
            style.set_property("position", "absolute")?;
            style.set_property("opacity", "0.001")?;
            style.set_property("pointer-events", "none")?;
            self.nodes.insert(
                node.id,
                DomNode {
                    element,
                    semantics: node.semantics.clone(),
                    bounds: None,
                    _handlers: handlers,
                },
            );
        }
        let current = self.nodes.get_mut(&node.id).expect("upserted DOM node");
        if current.semantics != node.semantics {
            apply_attributes(&current.element, node, &self.root.id())?;
            current.semantics = node.semantics.clone();
        }
        Ok(())
    }

    fn attach_children(&mut self, tree: &SemanticTree) -> Result<(), JsValue> {
        let scale = self
            .canvas
            .owner_document()
            .and_then(|document| document.default_view())
            .map_or(1.0, |window| window.device_pixel_ratio());
        for (id, bounds) in tree.parent_relative_bounds() {
            if let Some(node) = self.nodes.get_mut(&id) {
                apply_bounds(
                    node.element.unchecked_ref::<HtmlElement>(),
                    &mut node.bounds,
                    [
                        f64::from(bounds.origin.x) / scale,
                        f64::from(bounds.origin.y) / scale,
                        (f64::from(bounds.size.width) / scale).max(1.0),
                        (f64::from(bounds.size.height) / scale).max(1.0),
                    ],
                )?;
            }
        }
        let root = self
            .nodes
            .get(&tree.root)
            .map_or_else(|| self.root.clone().into(), |node| node.element.clone());
        if root.parent_node().as_ref() != Some(self.root.as_ref()) {
            self.root.append_child(&root)?;
        }
        for node in &tree.nodes {
            let Some(parent) = self.nodes.get(&node.id) else {
                continue;
            };
            let mut cursor = parent.element.first_child();
            for child in &node.children {
                if let Some(child) = self.nodes.get(child) {
                    if cursor.as_ref() == Some(child.element.as_ref()) {
                        cursor = child.element.next_sibling();
                    } else {
                        parent
                            .element
                            .insert_before(&child.element, cursor.as_ref())?;
                    }
                }
            }
        }
        Ok(())
    }

    fn focus(&self, id: SemanticNodeId) {
        if let Some(node) = self.nodes.get(&id)
            && let Some(element) = node.element.dyn_ref::<HtmlElement>()
        {
            // Keep canvas input uninterrupted during pointer/keyboard navigation.
            // Moving DOM focus here emits a winit blur and cancels the pressed button.
            let _ = self
                .canvas
                .set_attribute("aria-activedescendant", &element.id());
            let active = self
                .canvas
                .owner_document()
                .and_then(|document| document.active_element());
            if active
                .as_ref()
                .is_some_and(|active| self.root.contains(Some(active)))
            {
                // Preserve direct DOM navigation initiated by assistive technology.
                let _ = element.focus();
            }
        } else {
            let _ = self.canvas.remove_attribute("aria-activedescendant");
        }
    }
}

impl Drop for DomTree {
    fn drop(&mut self) {
        let _ = self.canvas.remove_attribute("aria-activedescendant");
        let _ = self.canvas.remove_attribute("aria-owns");
        self.root.remove();
    }
}

fn handlers(
    id: SemanticNodeId,
    actions: &[SemanticAction],
    element: &Element,
    callback: std::rc::Rc<dyn Fn(SemanticRequest)>,
) -> Result<Vec<EventHandler>, JsValue> {
    let mut output = Vec::new();
    for (name, action) in [
        ("click", SemanticAction::Click),
        ("focus", SemanticAction::Focus),
    ] {
        if !actions.contains(&action) {
            continue;
        }
        let callback = std::rc::Rc::clone(&callback);
        let handler = Closure::new(move |_event: Event| {
            callback(SemanticRequest {
                target: id,
                action,
                value: None,
            });
        });
        element.add_event_listener_with_callback(name, handler.as_ref().unchecked_ref())?;
        output.push(handler);
    }
    if actions.contains(&SemanticAction::SetValue)
        && (element.dyn_ref::<HtmlInputElement>().is_some()
            || element.dyn_ref::<HtmlTextAreaElement>().is_some())
    {
        let callback = std::rc::Rc::clone(&callback);
        let control = element.clone();
        let handler = Closure::new(move |_event: Event| {
            let value = control
                .dyn_ref::<HtmlInputElement>()
                .map(HtmlInputElement::value)
                .or_else(|| {
                    control
                        .dyn_ref::<HtmlTextAreaElement>()
                        .map(HtmlTextAreaElement::value)
                })
                .unwrap_or_default();
            callback(SemanticRequest {
                target: id,
                action: SemanticAction::SetValue,
                value: Some(SemanticValue::Text(value)),
            });
        });
        element.add_event_listener_with_callback("input", handler.as_ref().unchecked_ref())?;
        output.push(handler);
    }
    if actions.contains(&SemanticAction::Increment) || actions.contains(&SemanticAction::Decrement)
    {
        let callback = std::rc::Rc::clone(&callback);
        let increment = actions.contains(&SemanticAction::Increment);
        let decrement = actions.contains(&SemanticAction::Decrement);
        let handler = Closure::new(move |event: Event| {
            let Some(event) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            let action = match event.key().as_str() {
                "ArrowRight" | "ArrowUp" if increment => Some(SemanticAction::Increment),
                "ArrowLeft" | "ArrowDown" if decrement => Some(SemanticAction::Decrement),
                _ => None,
            };
            if let Some(action) = action {
                event.prevent_default();
                callback(SemanticRequest {
                    target: id,
                    action,
                    value: None,
                });
            }
        });
        element.add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())?;
        output.push(handler);
    }
    Ok(output)
}
