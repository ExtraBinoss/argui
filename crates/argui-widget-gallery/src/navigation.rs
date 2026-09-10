#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Page {
    #[default]
    Button,
    Badge,
    Card,
    Alert,
    Separator,
    Collapsible,
    Avatar,
    Empty,
    Kbd,
    Progress,
    AspectRatio,
    Input,
    TextArea,
    Checkbox,
    Switch,
    RadioGroup,
    Slider,
    Tabs,
    Select,
    Dialog,
    List,
    VList,
    Table,
    DataTable,
    Calendar,
    DatePicker,
    Toast,
    Menu,
    ContextMenu,
    Menubar,
    Layout,
    Motion,
    Effects,
    Typography,
    WebView,
    AsyncTasks,
    Actions,
    Editing,
    CustomTimeline,
}

impl Page {
    pub const ALL: [Self; 39] = [
        Self::Button,
        Self::Badge,
        Self::Card,
        Self::Alert,
        Self::Separator,
        Self::Collapsible,
        Self::Avatar,
        Self::Empty,
        Self::Kbd,
        Self::Progress,
        Self::AspectRatio,
        Self::Input,
        Self::TextArea,
        Self::Checkbox,
        Self::Switch,
        Self::RadioGroup,
        Self::Slider,
        Self::Tabs,
        Self::Select,
        Self::Dialog,
        Self::List,
        Self::VList,
        Self::Table,
        Self::DataTable,
        Self::Calendar,
        Self::DatePicker,
        Self::Toast,
        Self::Menu,
        Self::ContextMenu,
        Self::Menubar,
        Self::Layout,
        Self::Motion,
        Self::Effects,
        Self::Typography,
        Self::WebView,
        Self::AsyncTasks,
        Self::Actions,
        Self::Editing,
        Self::CustomTimeline,
    ];

    pub const fn category(self) -> &'static str {
        match self {
            Self::Button
            | Self::Badge
            | Self::Card
            | Self::Alert
            | Self::Separator
            | Self::Collapsible
            | Self::Avatar
            | Self::Empty
            | Self::Kbd
            | Self::Progress
            | Self::AspectRatio
            | Self::Input
            | Self::TextArea
            | Self::Checkbox
            | Self::Switch
            | Self::RadioGroup
            | Self::Slider
            | Self::Tabs
            | Self::Select
            | Self::Dialog
            | Self::List
            | Self::VList
            | Self::DataTable
            | Self::Calendar
            | Self::DatePicker
            | Self::Toast
            | Self::Menu
            | Self::ContextMenu
            | Self::Menubar
            | Self::Table => "Widgets",
            Self::Layout
            | Self::Motion
            | Self::Effects
            | Self::Typography
            | Self::WebView
            | Self::AsyncTasks
            | Self::Actions
            | Self::Editing
            | Self::CustomTimeline => "Examples",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Badge => "Badge",
            Self::Card => "Card",
            Self::Alert => "Alert",
            Self::Separator => "Separator",
            Self::Collapsible => "Collapsible",
            Self::Avatar => "Avatar",
            Self::Empty => "Empty",
            Self::Kbd => "Kbd",
            Self::Progress => "Progress",
            Self::AspectRatio => "Aspect ratio",
            Self::Input => "Input & Search",
            Self::TextArea => "Text area",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::RadioGroup => "Radio group",
            Self::Slider => "Slider",
            Self::Tabs => "Tabs",
            Self::Select => "Select",
            Self::Dialog => "Dialog",
            Self::List => "List",
            Self::VList => "VList",
            Self::Table => "Table",
            Self::DataTable => "Data table",
            Self::Calendar => "Calendar",
            Self::DatePicker => "Date picker",
            Self::Toast => "Toast",
            Self::Menu => "Menu",
            Self::ContextMenu => "Context menu",
            Self::Menubar => "Menubar",
            Self::Layout => "Web layout",
            Self::Motion => "Motion & loading",
            Self::Effects => "GPU effects / WGSL",
            Self::Typography => "Typography & selection",
            Self::WebView => "WebView",
            Self::AsyncTasks => "Async tasks",
            Self::Actions => "Actions",
            Self::Editing => "Editing & Password",
            Self::CustomTimeline => "Custom Timeline",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Badge => "badge",
            Self::Card => "card",
            Self::Alert => "alert",
            Self::Separator => "separator",
            Self::Collapsible => "collapsible",
            Self::Avatar => "avatar",
            Self::Empty => "empty",
            Self::Kbd => "kbd",
            Self::Progress => "progress",
            Self::AspectRatio => "aspect-ratio",
            Self::Input => "input",
            Self::TextArea => "textarea",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::RadioGroup => "radio-group",
            Self::Slider => "slider",
            Self::Tabs => "tabs",
            Self::Select => "select",
            Self::Dialog => "dialog",
            Self::List => "list",
            Self::VList => "vlist",
            Self::Table => "table",
            Self::DataTable => "data-table",
            Self::Calendar => "calendar",
            Self::DatePicker => "date-picker",
            Self::Toast => "toast",
            Self::Menu => "menu",
            Self::ContextMenu => "context-menu",
            Self::Menubar => "menubar",
            Self::Layout => "layout",
            Self::Motion => "motion",
            Self::Effects => "effects",
            Self::Typography => "typography",
            Self::WebView => "webview",
            Self::AsyncTasks => "async-tasks",
            Self::Actions => "actions",
            Self::Editing => "editing",
            Self::CustomTimeline => "custom-timeline",
        }
    }

    pub fn matches(self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        query.is_empty()
            || self.label().to_lowercase().contains(&query)
            || self.category().to_lowercase().contains(&query)
    }

    pub fn from_navigation_key(key: &str) -> Option<Self> {
        let slug = key.strip_prefix("nav::")?;
        Self::ALL.into_iter().find(|page| page.slug() == slug)
    }
}
