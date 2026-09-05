use accesskit::{
    Action, ActionRequest, Live, Node, NodeId, Orientation as AccessOrientation, Rect,
    Role as AccessRole, Tree, TreeId, TreeUpdate,
};

use crate::{
    LiveRegion, Role, SemanticAction, SemanticNode, SemanticNodeId, SemanticPatch, SemanticRequest,
    SemanticTree, SemanticValue,
};

impl TryFrom<ActionRequest> for SemanticRequest {
    type Error = ();

    fn try_from(request: ActionRequest) -> Result<Self, Self::Error> {
        let action = match request.action {
            Action::Click => SemanticAction::Click,
            Action::Focus => SemanticAction::Focus,
            Action::Blur => SemanticAction::Blur,
            Action::Increment => SemanticAction::Increment,
            Action::Decrement => SemanticAction::Decrement,
            Action::Expand => SemanticAction::Expand,
            Action::Collapse => SemanticAction::Collapse,
            Action::SetValue => SemanticAction::SetValue,
            Action::ScrollIntoView => SemanticAction::ScrollIntoView,
            _ => return Err(()),
        };
        let value = match request.data {
            Some(accesskit::ActionData::Value(value)) => {
                Some(SemanticValue::Text(value.into_string()))
            }
            Some(accesskit::ActionData::NumericValue(value)) => Some(SemanticValue::Number {
                value,
                minimum: None,
                maximum: None,
                step: None,
            }),
            _ => None,
        };
        Ok(Self {
            target: SemanticNodeId::new(request.target_node.0),
            action,
            value,
        })
    }
}

pub struct AccessKitTree;

impl AccessKitTree {
    #[must_use]
    pub fn full(tree: &SemanticTree) -> TreeUpdate {
        let mut metadata = Tree::new(NodeId(tree.root.get()));
        metadata.toolkit_name = Some("Argui".into());
        metadata.toolkit_version = Some(env!("CARGO_PKG_VERSION").into());
        TreeUpdate {
            nodes: tree.nodes.iter().map(lower_node).collect(),
            tree: Some(metadata),
            tree_id: TreeId::ROOT,
            focus: NodeId(tree.focus.get()),
        }
    }

    #[must_use]
    pub fn patch(patch: &SemanticPatch, tree: &SemanticTree) -> TreeUpdate {
        TreeUpdate {
            nodes: patch.upserts.iter().map(lower_node).collect(),
            tree: patch.root.map(|root| Tree::new(NodeId(root.get()))),
            tree_id: TreeId::ROOT,
            focus: NodeId(tree.focus.get()),
        }
    }
}

fn lower_node(node: &SemanticNode) -> (NodeId, Node) {
    let mut output = Node::new(lower_role(node.semantics.role));
    output.set_bounds(Rect {
        x0: f64::from(node.bounds.origin.x),
        y0: f64::from(node.bounds.origin.y),
        x1: f64::from(node.bounds.origin.x + node.bounds.size.width),
        y1: f64::from(node.bounds.origin.y + node.bounds.size.height),
    });
    output.set_children(
        node.children
            .iter()
            .map(|id| NodeId(id.get()))
            .collect::<Vec<_>>(),
    );
    if let Some(label) = &node.semantics.label {
        output.set_label(label.clone());
    }
    if let Some(description) = &node.semantics.description {
        output.set_description(description.clone());
    }
    match &node.semantics.value {
        Some(SemanticValue::Text(value)) => output.set_value(value.clone()),
        Some(SemanticValue::Number {
            value,
            minimum,
            maximum,
            step,
        }) => {
            output.set_numeric_value(*value);
            if let Some(value) = minimum {
                output.set_min_numeric_value(*value);
            }
            if let Some(value) = maximum {
                output.set_max_numeric_value(*value);
            }
            if let Some(value) = step {
                output.set_numeric_value_step(*value);
            }
        }
        None => {}
    }
    for action in &node.semantics.actions {
        output.add_action(lower_action(*action));
    }
    if node.semantics.state.disabled {
        output.set_disabled();
    }
    output.set_selected(node.semantics.state.selected);
    if let Some(value) = node.semantics.state.checked {
        output.set_toggled(if value {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        });
    }
    if let Some(value) = node.semantics.state.expanded {
        output.set_expanded(value);
    }
    if node.semantics.state.required {
        output.set_required();
    }
    if node.semantics.state.read_only {
        output.set_read_only();
    }
    if node.semantics.state.invalid {
        output.set_invalid(accesskit::Invalid::True);
    }
    if node.semantics.state.modal {
        output.set_modal();
    }
    if node.semantics.state.busy {
        output.set_busy();
    }
    match node.semantics.live {
        LiveRegion::Off => {}
        LiveRegion::Polite => output.set_live(Live::Polite),
        LiveRegion::Assertive => output.set_live(Live::Assertive),
    }
    if let Some(level) = node.semantics.level {
        output.set_level(level as usize);
    }
    if let Some(position) = node.semantics.position_in_set {
        output.set_position_in_set(position as usize);
    }
    if let Some(size) = node.semantics.set_size {
        output.set_size_of_set(size as usize);
    }
    if let Some(orientation) = node.semantics.orientation {
        output.set_orientation(match orientation {
            crate::Orientation::Horizontal => AccessOrientation::Horizontal,
            crate::Orientation::Vertical => AccessOrientation::Vertical,
        });
    }
    (NodeId(node.id.get()), output)
}

const fn lower_role(role: Role) -> AccessRole {
    match role {
        Role::Generic => AccessRole::GenericContainer,
        Role::Window => AccessRole::Window,
        Role::Group => AccessRole::Group,
        Role::Text => AccessRole::Label,
        Role::Heading => AccessRole::Heading,
        Role::Image => AccessRole::Image,
        Role::Link => AccessRole::Link,
        Role::Button => AccessRole::Button,
        Role::CheckBox => AccessRole::CheckBox,
        Role::RadioButton => AccessRole::RadioButton,
        Role::Switch => AccessRole::Switch,
        Role::TextInput => AccessRole::TextInput,
        Role::TextArea => AccessRole::MultilineTextInput,
        Role::SearchInput => AccessRole::SearchInput,
        Role::List => AccessRole::List,
        Role::ListItem => AccessRole::ListItem,
        Role::ListBox => AccessRole::ListBox,
        Role::Option => AccessRole::ListBoxOption,
        Role::Menu => AccessRole::Menu,
        Role::MenuItem => AccessRole::MenuItem,
        Role::Slider => AccessRole::Slider,
        Role::Progress => AccessRole::ProgressIndicator,
        Role::Tab => AccessRole::Tab,
        Role::TabList => AccessRole::TabList,
        Role::TabPanel => AccessRole::TabPanel,
        Role::Dialog => AccessRole::Dialog,
        Role::Alert => AccessRole::Alert,
        Role::Separator => AccessRole::Splitter,
        Role::Tree => AccessRole::Tree,
        Role::TreeItem => AccessRole::TreeItem,
    }
}

const fn lower_action(action: SemanticAction) -> Action {
    match action {
        SemanticAction::Click => Action::Click,
        SemanticAction::Focus => Action::Focus,
        SemanticAction::Blur => Action::Blur,
        SemanticAction::Increment => Action::Increment,
        SemanticAction::Decrement => Action::Decrement,
        SemanticAction::Expand => Action::Expand,
        SemanticAction::Collapse => Action::Collapse,
        SemanticAction::SetValue => Action::SetValue,
        SemanticAction::ScrollIntoView => Action::ScrollIntoView,
    }
}
