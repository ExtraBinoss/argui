#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Role {
    #[default]
    Generic,
    Window,
    Group,
    Text,
    Heading,
    Image,
    Link,
    Button,
    CheckBox,
    RadioButton,
    Switch,
    TextInput,
    TextArea,
    SearchInput,
    Table,
    Grid,
    Row,
    ColumnHeader,
    Cell,
    List,
    ListItem,
    Tree,
    TreeItem,
    ListBox,
    Option,
    Menu,
    MenuItem,
    Slider,
    Progress,
    Tab,
    TabList,
    TabPanel,
    Dialog,
    Alert,
    Separator,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticAction {
    Click,
    Focus,
    Blur,
    Increment,
    Decrement,
    Expand,
    Collapse,
    SetValue,
    ScrollIntoView,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SemanticValue {
    Text(String),
    Number {
        value: f64,
        minimum: Option<f64>,
        maximum: Option<f64>,
        step: Option<f64>,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LiveRegion {
    #[default]
    Off,
    Polite,
    Assertive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticState {
    pub protected: bool,
    pub disabled: bool,
    pub selected: bool,
    pub multiselectable: bool,
    pub checked: Option<bool>,
    pub expanded: Option<bool>,
    pub required: bool,
    pub read_only: bool,
    pub invalid: bool,
    pub modal: bool,
    pub busy: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    pub role: Role,
    pub label: Option<String>,
    pub description: Option<String>,
    pub value: Option<SemanticValue>,
    pub state: SemanticState,
    pub actions: Vec<SemanticAction>,
    pub live: LiveRegion,
    pub orientation: Option<Orientation>,
    pub level: Option<u32>,
    pub position_in_set: Option<u32>,
    pub set_size: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticRequest {
    pub target: crate::SemanticNodeId,
    pub action: SemanticAction,
    pub value: Option<SemanticValue>,
}

impl Semantics {
    #[must_use]
    pub const fn new(role: Role) -> Self {
        Self {
            role,
            label: None,
            description: None,
            value: None,
            state: SemanticState {
                protected: false,
                disabled: false,
                selected: false,
                multiselectable: false,
                checked: None,
                expanded: None,
                required: false,
                read_only: false,
                invalid: false,
                modal: false,
                busy: false,
            },
            actions: Vec::new(),
            live: LiveRegion::Off,
            orientation: None,
            level: None,
            position_in_set: None,
            set_size: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub fn value(mut self, value: SemanticValue) -> Self {
        self.value = Some(value);
        self
    }

    #[must_use]
    pub fn action(mut self, action: SemanticAction) -> Self {
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
        self
    }

    #[must_use]
    pub fn state(mut self, state: SemanticState) -> Self {
        self.state = state;
        self
    }

    #[must_use]
    pub const fn live(mut self, live: LiveRegion) -> Self {
        self.live = live;
        self
    }

    #[must_use]
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    #[must_use]
    pub const fn level(mut self, level: u32) -> Self {
        self.level = Some(level);
        self
    }

    #[must_use]
    pub const fn position_in_set(mut self, position: u32, size: u32) -> Self {
        self.position_in_set = Some(position);
        self.set_size = Some(size);
        self
    }
}
