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
    Label,
    Skeleton,
    Breadcrumb,
    Pagination,
    Input,
    TextArea,
    Checkbox,
    Switch,
    RadioGroup,
    Slider,
    Tabs,
    Select,
    Dialog,
    Popover,
    Tooltip,
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
    pub const ALL: [Self; 45] = [
        Self::Alert,
        Self::AspectRatio,
        Self::Avatar,
        Self::Badge,
        Self::Breadcrumb,
        Self::Button,
        Self::Calendar,
        Self::Card,
        Self::Checkbox,
        Self::Collapsible,
        Self::ContextMenu,
        Self::DataTable,
        Self::DatePicker,
        Self::Dialog,
        Self::Empty,
        Self::Input,
        Self::Kbd,
        Self::Label,
        Self::List,
        Self::Menu,
        Self::Menubar,
        Self::Pagination,
        Self::Popover,
        Self::Progress,
        Self::RadioGroup,
        Self::Select,
        Self::Separator,
        Self::Skeleton,
        Self::Slider,
        Self::Switch,
        Self::Table,
        Self::Tabs,
        Self::TextArea,
        Self::Toast,
        Self::Tooltip,
        Self::VList,
        Self::Actions,
        Self::AsyncTasks,
        Self::CustomTimeline,
        Self::Editing,
        Self::Effects,
        Self::Motion,
        Self::Typography,
        Self::Layout,
        Self::WebView,
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
            | Self::Label
            | Self::Skeleton
            | Self::Breadcrumb
            | Self::Pagination
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
            | Self::Popover
            | Self::Tooltip
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
            Self::Label => "Label",
            Self::Skeleton => "Skeleton",
            Self::Breadcrumb => "Breadcrumb",
            Self::Pagination => "Pagination",
            Self::Input => "Input & Search",
            Self::TextArea => "Text area",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::RadioGroup => "Radio group",
            Self::Slider => "Slider",
            Self::Tabs => "Tabs",
            Self::Select => "Select",
            Self::Dialog => "Dialog",
            Self::Popover => "Popover",
            Self::Tooltip => "Tooltip",
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
            Self::Label => "label",
            Self::Skeleton => "skeleton",
            Self::Breadcrumb => "breadcrumb",
            Self::Pagination => "pagination",
            Self::Input => "input",
            Self::TextArea => "textarea",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::RadioGroup => "radio-group",
            Self::Slider => "slider",
            Self::Tabs => "tabs",
            Self::Select => "select",
            Self::Dialog => "dialog",
            Self::Popover => "popover",
            Self::Tooltip => "tooltip",
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

impl Page {
    pub const fn description(self) -> &'static str {
        match self {
            Self::List => "Selection and keyboard navigation.",
            Self::VList => "Measured variable-height rows and virtual scrolling.",
            Self::Table => "Columns and row selection.",
            Self::Menu => "Add notes, toggle details and choose a comfortable or compact layout.",
            Self::ContextMenu => "Open a menu at the pointer or with Shift+F10.",
            Self::Menubar => {
                "File manages your notebook; View changes its appearance. Try the arrow keys."
            }
            Self::DataTable => {
                "Sort tasks, select rows and edit Hours with Enter. Drag a column edge to resize."
            }
            Self::Calendar => "Choose dates with arrows and PageUp/PageDown.",
            Self::DatePicker => "Type a date or choose it from the calendar.",
            Self::Toast => "Notifications with a bounded queue and explicit dismissal.",
            Self::Avatar => "Images and initials with a shared accessible name.",
            Self::Empty => "Give an empty view a useful message and a next step.",
            Self::Kbd => "Keycaps and shortcut combinations.",
            Self::AspectRatio => "Keep content at a consistent width-to-height ratio.",
            Self::Progress => {
                "Determinate and indeterminate progress, with reduced-motion support."
            }
            Self::Label => "Clear labels associated with their controls.",
            Self::Skeleton => "Soft loading shapes that respect reduced motion.",
            Self::Breadcrumb => "A trail of links back to the current page’s ancestors.",
            Self::Pagination => "Page navigation with a compact range and clear boundaries.",
            Self::Badge => "Compact labels with variants and optional icons.",
            Self::Card => "A surface with a title, description, content, action and footer.",
            Self::Alert => "Inline messages with accessible announcements.",
            Self::Separator => "Horizontal and vertical dividers, with optional centered labels.",
            Self::Collapsible => "Show and hide content with a button, Enter or Space.",
            Self::Button => "Actions with variants, icons, loading and accessible activation.",
            Self::Input => "Controlled single-line and search fields.",
            Self::TextArea => "Multiline editing, scrolling, clipping and resize capture.",
            Self::Checkbox => {
                "Unchecked, checked and mixed states with keyboard and touch activation."
            }
            Self::Switch => "Animated binary preferences.",
            Self::RadioGroup => "Exclusive selection with semantic grouping.",
            Self::Slider => "Pointer, touch, keyboard and accessibility values.",
            Self::Tabs => "Roving navigation and one mounted panel.",
            Self::Select => "Anchored, collision-aware option overlay.",
            Self::Dialog => "Modal focus containment and restoration.",
            Self::Popover => "Interactive panels with solid, blurred and custom effect surfaces.",
            Self::Tooltip => {
                "Helpful descriptions on hover or keyboard focus, with customizable surfaces."
            }
            Self::Layout => "CSS-shaped Block, Flex, Grid, box model and text alignment.",
            Self::Motion => "Frame-paced feedback and interaction transitions.",
            Self::Effects => "Custom WGSL through the generic effect registry.",
            Self::Typography => "Rich spans, decoration, clamping and web-like text selection.",
            Self::WebView => {
                "Retained web content, with separate email and webpage security policies."
            }
            Self::AsyncTasks => "Owned, cancellable work with event-driven delivery to the UI.",
            Self::Actions => "One command for buttons, menus, palettes and focused shortcuts.",
            Self::Editing => {
                "Transactional undo/redo, Unicode, filtered fields and protected passwords."
            }
            Self::CustomTimeline => {
                "Custom measurement and painting with draggable clips and standard controls."
            }
        }
    }
}
