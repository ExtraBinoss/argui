use std::collections::HashMap;

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Element, Event, HtmlCanvasElement, HtmlElement, HtmlInputElement, KeyboardEvent};

use crate::{
    LiveRegion, Role, SemanticAction, SemanticNode, SemanticNodeId, SemanticPatch, SemanticRequest,
    SemanticTree, SemanticValue,
};

struct DomNode {
    element: Element,
    actions: Vec<SemanticAction>,
    _handlers: Vec<Closure<dyn FnMut(Event)>>,
}

pub struct DomTree {
    canvas: HtmlCanvasElement,
    root: HtmlElement,
    nodes: HashMap<SemanticNodeId, DomNode>,
    snapshot: SemanticTree,
    on_action: std::rc::Rc<dyn Fn(SemanticRequest)>,
}

impl DomTree {
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

    fn position_root(&self) -> Result<(), JsValue> {
        let rect = self.canvas.get_bounding_client_rect();
        let style = self.root.style();
        style.set_property("left", &format!("{}px", rect.left()))?;
        style.set_property("top", &format!("{}px", rect.top()))?;
        style.set_property("width", &format!("{}px", rect.width()))?;
        style.set_property("height", &format!("{}px", rect.height()))?;
        Ok(())
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
                || current.actions != node.semantics.actions
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
            let handlers = handlers(
                node.id,
                &node.semantics.actions,
                &element,
                std::rc::Rc::clone(&self.on_action),
            )?;
            self.nodes.insert(
                node.id,
                DomNode {
                    element,
                    actions: node.semantics.actions.clone(),
                    _handlers: handlers,
                },
            );
        }
        let current = &self.nodes[&node.id].element;
        apply_attributes(current, node, self.canvas.width(), self.canvas.height())
    }

    fn attach_children(&self, tree: &SemanticTree) -> Result<(), JsValue> {
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
            for child in &node.children {
                if let Some(child) = self.nodes.get(child) {
                    parent.element.append_child(&child.element)?;
                }
            }
        }
        Ok(())
    }

    fn focus(&self, id: SemanticNodeId) {
        if let Some(node) = self.nodes.get(&id)
            && let Some(element) = node.element.dyn_ref::<HtmlElement>()
        {
            let _ = element.focus();
        }
    }
}

impl Drop for DomTree {
    fn drop(&mut self) {
        self.root.remove();
    }
}

fn handlers(
    id: SemanticNodeId,
    actions: &[SemanticAction],
    element: &Element,
    callback: std::rc::Rc<dyn Fn(SemanticRequest)>,
) -> Result<Vec<Closure<dyn FnMut(Event)>>, JsValue> {
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
        && let Some(input) = element.dyn_ref::<HtmlInputElement>()
    {
        let callback = std::rc::Rc::clone(&callback);
        let input = input.clone();
        let handler = Closure::new(move |_event: Event| {
            callback(SemanticRequest {
                target: id,
                action: SemanticAction::SetValue,
                value: Some(SemanticValue::Text(input.value())),
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

fn apply_attributes(
    element: &Element,
    node: &SemanticNode,
    canvas_width: u32,
    canvas_height: u32,
) -> Result<(), JsValue> {
    element.set_attribute("data-argui-node", &node.id.get().to_string())?;
    element.set_attribute("role", aria_role(node.semantics.role))?;
    if node.semantics.actions.contains(&SemanticAction::Focus) {
        element.set_attribute("tabindex", "0")?;
    } else {
        element.remove_attribute("tabindex")?;
    }
    match node.semantics.role {
        Role::Button => element.set_attribute("type", "button")?,
        Role::SearchInput => element.set_attribute("type", "search")?,
        _ => element.remove_attribute("type")?,
    }
    set_optional(element, "aria-label", node.semantics.label.as_deref())?;
    set_optional(
        element,
        "aria-description",
        node.semantics.description.as_deref(),
    )?;
    element.set_attribute(
        "aria-disabled",
        if node.semantics.state.disabled {
            "true"
        } else {
            "false"
        },
    )?;
    set_bool(element, "aria-selected", node.semantics.state.selected)?;
    set_bool(element, "aria-required", node.semantics.state.required)?;
    set_bool(element, "aria-readonly", node.semantics.state.read_only)?;
    set_bool(element, "aria-invalid", node.semantics.state.invalid)?;
    set_optional(
        element,
        "aria-modal",
        node.semantics.state.modal.then_some("true"),
    )?;
    set_optional_bool(element, "aria-checked", node.semantics.state.checked)?;
    set_optional_bool(element, "aria-expanded", node.semantics.state.expanded)?;
    for attribute in [
        "aria-valuetext",
        "aria-valuenow",
        "aria-valuemin",
        "aria-valuemax",
    ] {
        element.remove_attribute(attribute)?;
    }
    match &node.semantics.value {
        Some(SemanticValue::Text(value)) => {
            if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                if input.value() != *value {
                    input.set_value(value);
                }
            } else {
                element.set_attribute("aria-valuetext", value)?;
            }
        }
        Some(SemanticValue::Number {
            value,
            minimum,
            maximum,
            step: _,
        }) => {
            element.set_attribute("aria-valuenow", &value.to_string())?;
            set_optional_number(element, "aria-valuemin", *minimum)?;
            set_optional_number(element, "aria-valuemax", *maximum)?;
        }
        None => {}
    }
    set_optional(
        element,
        "aria-live",
        match node.semantics.live {
            LiveRegion::Off => None,
            LiveRegion::Polite => Some("polite"),
            LiveRegion::Assertive => Some("assertive"),
        },
    )?;
    set_optional(
        element,
        "aria-orientation",
        node.semantics
            .orientation
            .map(|orientation| match orientation {
                crate::Orientation::Horizontal => "horizontal",
                crate::Orientation::Vertical => "vertical",
            }),
    )?;
    set_optional_number(element, "aria-level", node.semantics.level.map(f64::from))?;
    set_optional_number(
        element,
        "aria-posinset",
        node.semantics.position_in_set.map(f64::from),
    )?;
    set_optional_number(
        element,
        "aria-setsize",
        node.semantics.set_size.map(f64::from),
    )?;
    let scale_x = f64::from(canvas_width.max(1));
    let scale_y = f64::from(canvas_height.max(1));
    let canvas_rect = element
        .owner_document()
        .and_then(|document| document.default_view())
        .map_or(1.0, |window| window.device_pixel_ratio());
    let x = f64::from(node.bounds.origin.x) / canvas_rect;
    let y = f64::from(node.bounds.origin.y) / canvas_rect;
    let width = f64::from(node.bounds.size.width) / canvas_rect;
    let height = f64::from(node.bounds.size.height) / canvas_rect;
    let style = element.dyn_ref::<HtmlElement>().map(HtmlElement::style);
    if let Some(style) = style {
        style.set_property("position", "absolute")?;
        style.set_property("left", &format!("{}px", x.min(scale_x)))?;
        style.set_property("top", &format!("{}px", y.min(scale_y)))?;
        style.set_property("width", &format!("{}px", width.max(1.0)))?;
        style.set_property("height", &format!("{}px", height.max(1.0)))?;
        style.set_property("opacity", "0.001")?;
        style.set_property("pointer-events", "none")?;
    }
    Ok(())
}

fn set_optional(element: &Element, name: &str, value: Option<&str>) -> Result<(), JsValue> {
    if let Some(value) = value {
        element.set_attribute(name, value)
    } else {
        element.remove_attribute(name)
    }
}

fn set_bool(element: &Element, name: &str, value: bool) -> Result<(), JsValue> {
    element.set_attribute(name, if value { "true" } else { "false" })
}

fn set_optional_bool(element: &Element, name: &str, value: Option<bool>) -> Result<(), JsValue> {
    set_optional(
        element,
        name,
        value.map(|value| if value { "true" } else { "false" }),
    )
}

fn set_optional_number(element: &Element, name: &str, value: Option<f64>) -> Result<(), JsValue> {
    if let Some(value) = value {
        element.set_attribute(name, &value.to_string())
    } else {
        element.remove_attribute(name)
    }
}

const fn html_tag(role: Role) -> &'static str {
    match role {
        Role::Button => "button",
        Role::TextInput | Role::SearchInput => "input",
        Role::Link => "a",
        Role::List => "ul",
        Role::ListItem => "li",
        Role::Heading => "h2",
        Role::Text => "span",
        _ => "div",
    }
}

const fn aria_role(role: Role) -> &'static str {
    match role {
        Role::Generic => "presentation",
        Role::Window => "application",
        Role::Group => "group",
        Role::Text => "text",
        Role::Heading => "heading",
        Role::Image => "img",
        Role::Link => "link",
        Role::Button => "button",
        Role::CheckBox => "checkbox",
        Role::RadioButton => "radio",
        Role::Switch => "switch",
        Role::TextInput => "textbox",
        Role::SearchInput => "searchbox",
        Role::List => "list",
        Role::ListItem => "listitem",
        Role::ListBox => "listbox",
        Role::Option => "option",
        Role::Menu => "menu",
        Role::MenuItem => "menuitem",
        Role::Slider => "slider",
        Role::Progress => "progressbar",
        Role::Tab => "tab",
        Role::TabList => "tablist",
        Role::TabPanel => "tabpanel",
        Role::Dialog => "dialog",
        Role::Alert => "alert",
        Role::Separator => "separator",
    }
}
