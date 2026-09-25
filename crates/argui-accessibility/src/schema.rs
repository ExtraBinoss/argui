#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Role {
    #[default]
    Generic,
    Window,
    Group,
    Navigation,
    Text,
    Heading,
    Image,
    Link,
    Button,
    CheckBox,
    RadioButton,
    RadioGroup,
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
    MenuBar,
    MenuItemCheckBox,
    MenuItemRadio,
    ComboBox,
    Tooltip,
    Status,
    AlertDialog,
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

/// Meaning of the current item within a related set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Current {
    True,
    Page,
    Step,
    Location,
    Date,
    Time,
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
    pub current: Option<Current>,
    pub multiselectable: bool,
    pub checked: Option<CheckedState>,
    /// Persistent pressed state of a toggle button, independent from pointer press.
    pub pressed: Option<bool>,
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
    pub relations: crate::SemanticRelations,
    pub grid: crate::GridPosition,
    pub sort: Option<crate::SortDirection>,
    pub popup: Option<crate::PopupKind>,
    pub focus_policy: FocusPolicy,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticRequest {
    pub target: crate::SemanticNodeId,
    pub action: SemanticAction,
    pub value: Option<SemanticValue>,
}

impl Semantics {
    /// Creates semantics for `role`, with optional metadata unset and default state.
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
                current: None,
                multiselectable: false,
                checked: None,
                pressed: None,
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
            relations: crate::SemanticRelations::new(),
            grid: crate::GridPosition {
                row_count: None,
                column_count: None,
                row_index: None,
                column_index: None,
            },
            sort: None,
            popup: None,
            focus_policy: FocusPolicy::None,
        }
    }

    /// Sets the accessible label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the accessible description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the semantic value exposed by this element.
    #[must_use]
    pub fn value(mut self, value: SemanticValue) -> Self {
        self.value = Some(value);
        self
    }

    /// Adds an available action if it is not already present.
    #[must_use]
    pub fn action(mut self, action: SemanticAction) -> Self {
        if !self.actions.contains(&action) {
            self.actions.push(action);
        }
        self
    }

    /// Replaces the element's semantic state.
    #[must_use]
    pub fn state(mut self, state: SemanticState) -> Self {
        self.state = state;
        self
    }

    /// Sets the live-region announcement behavior.
    #[must_use]
    pub const fn live(mut self, live: LiveRegion) -> Self {
        self.live = live;
        self
    }

    /// Sets the element's layout orientation.
    #[must_use]
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Sets the heading level.
    #[must_use]
    pub const fn level(mut self, level: u32) -> Self {
        self.level = Some(level);
        self
    }

    /// Sets the one-based position and total size of this item in its set.
    ///
    /// # Arguments
    /// * `position` — one-based position within the set.
    /// * `size` — total number of items in the set.
    #[must_use]
    pub const fn position_in_set(mut self, position: u32, size: u32) -> Self {
        self.position_in_set = Some(position);
        self.set_size = Some(size);
        self
    }
}

/// An absent checked value is distinct from a present mixed selection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckedState {
    #[default]
    Unchecked,
    Checked,
    Mixed,
}

impl CheckedState {
    /// Toggles unchecked to checked, and checked to unchecked; mixed becomes checked.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Checked => Self::Unchecked,
            Self::Unchecked | Self::Mixed => Self::Checked,
        }
    }
}

/// Whether an element can receive focus and participate in sequential navigation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FocusPolicy {
    #[default]
    None,
    Programmatic,
    TabStop,
}

impl FocusPolicy {
    /// Returns whether this policy permits programmatic or sequential focus.
    #[must_use]
    pub const fn is_focusable(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns whether this policy participates in sequential tab navigation.
    #[must_use]
    pub const fn is_tab_stop(self) -> bool {
        matches!(self, Self::TabStop)
    }
}
