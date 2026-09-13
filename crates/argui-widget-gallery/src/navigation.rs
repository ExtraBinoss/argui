#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Page {
    #[default]
    Button,
    Accordion,
    AlertDialog,
    Attachment,
    Bubble,
    ButtonGroup,
    Carousel,
    Chart,
    Combobox,
    Direction,
    Drawer,
    Field,
    HoverCard,
    InputGroup,
    InputOtp,
    Item,
    Marker,
    Message,
    MessageScroller,
    NativeSelect,
    NavigationMenu,
    Questionnaire,
    ScrollArea,
    Sheet,
    Sidebar,
    Toggle,
    ToggleGroup,

    Badge,
    Card,
    Alert,
    Separator,
    Collapsible,
    Avatar,
    Empty,
    FilePicker,
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
    ColorPicker,
    AnimatedText,
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
    LiquidGlass,
    ScrollShadow,
    Typography,
    WebView,
    AsyncTasks,
    Actions,
    Editing,
    CustomTimeline,
}

impl Page {
    pub const ALL: [Self; 75] = [
        Self::Accordion,
        Self::Alert,
        Self::AlertDialog,
        Self::AnimatedText,
        Self::AspectRatio,
        Self::Attachment,
        Self::Avatar,
        Self::Badge,
        Self::Breadcrumb,
        Self::Bubble,
        Self::Button,
        Self::ButtonGroup,
        Self::Calendar,
        Self::Card,
        Self::Carousel,
        Self::Chart,
        Self::Checkbox,
        Self::Collapsible,
        Self::ColorPicker,
        Self::Combobox,
        Self::ContextMenu,
        Self::DataTable,
        Self::DatePicker,
        Self::Dialog,
        Self::Direction,
        Self::Drawer,
        Self::Empty,
        Self::Field,
        Self::FilePicker,
        Self::HoverCard,
        Self::Input,
        Self::InputGroup,
        Self::InputOtp,
        Self::Item,
        Self::Kbd,
        Self::Label,
        Self::List,
        Self::Marker,
        Self::Menu,
        Self::Menubar,
        Self::Message,
        Self::MessageScroller,
        Self::NativeSelect,
        Self::NavigationMenu,
        Self::Pagination,
        Self::Popover,
        Self::Progress,
        Self::Questionnaire,
        Self::RadioGroup,
        Self::ScrollArea,
        Self::Select,
        Self::Separator,
        Self::Sheet,
        Self::Sidebar,
        Self::Skeleton,
        Self::Slider,
        Self::Switch,
        Self::Table,
        Self::Tabs,
        Self::TextArea,
        Self::Toast,
        Self::Toggle,
        Self::ToggleGroup,
        Self::Tooltip,
        Self::VList,
        Self::LiquidGlass,
        Self::ScrollShadow,
        Self::Actions,
        Self::AsyncTasks,
        Self::CustomTimeline,
        Self::Editing,
        Self::Motion,
        Self::Typography,
        Self::Layout,
        Self::WebView,
    ];

    pub const fn category(self) -> &'static str {
        match self {
            Self::Accordion
            | Self::AlertDialog
            | Self::Attachment
            | Self::Bubble
            | Self::ButtonGroup
            | Self::Carousel
            | Self::Chart
            | Self::Combobox
            | Self::Direction
            | Self::Drawer
            | Self::Field
            | Self::HoverCard
            | Self::InputGroup
            | Self::InputOtp
            | Self::Item
            | Self::Marker
            | Self::Message
            | Self::MessageScroller
            | Self::NativeSelect
            | Self::NavigationMenu
            | Self::Questionnaire
            | Self::ScrollArea
            | Self::Sheet
            | Self::Sidebar
            | Self::Toggle
            | Self::ToggleGroup
            | Self::Button
            | Self::Badge
            | Self::Card
            | Self::Alert
            | Self::Separator
            | Self::Collapsible
            | Self::Avatar
            | Self::Empty
            | Self::FilePicker
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
            | Self::AnimatedText
            | Self::ColorPicker
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
            Self::LiquidGlass | Self::ScrollShadow => "Effects",
            Self::Layout
            | Self::Motion
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
            Self::Accordion => "Accordion",
            Self::AlertDialog => "Alert dialog",
            Self::Attachment => "Attachment",
            Self::Bubble => "Bubble",
            Self::ButtonGroup => "Button group",
            Self::Carousel => "Carousel",
            Self::Chart => "Chart",
            Self::Combobox => "Combobox",
            Self::Direction => "Direction",
            Self::Drawer => "Drawer",
            Self::Field => "Field",
            Self::HoverCard => "Hover card",
            Self::InputGroup => "Input group",
            Self::InputOtp => "Input OTP",
            Self::Item => "Item",
            Self::Marker => "Marker",
            Self::Message => "Message",
            Self::MessageScroller => "Message scroller",
            Self::NativeSelect => "Native select",
            Self::NavigationMenu => "Navigation menu",
            Self::Questionnaire => "Questionnaire",
            Self::ScrollArea => "Scroll area",
            Self::Sheet => "Sheet",
            Self::Sidebar => "Sidebar",
            Self::Toggle => "Toggle",
            Self::ToggleGroup => "Toggle group",

            Self::Button => "Button",
            Self::Badge => "Badge",
            Self::Card => "Card",
            Self::Alert => "Alert",
            Self::Separator => "Separator",
            Self::Collapsible => "Collapsible",
            Self::Avatar => "Avatar",
            Self::Empty => "Empty",
            Self::FilePicker => "File picker",
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
            Self::AnimatedText => "Animated text",
            Self::ColorPicker => "Color picker",
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
            Self::LiquidGlass => "Liquid glass",
            Self::ScrollShadow => "Scroll shadow",
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
            Self::Accordion => "accordion",
            Self::AlertDialog => "alert-dialog",
            Self::Attachment => "attachment",
            Self::Bubble => "bubble",
            Self::ButtonGroup => "button-group",
            Self::Carousel => "carousel",
            Self::Chart => "chart",
            Self::Combobox => "combobox",
            Self::Direction => "direction",
            Self::Drawer => "drawer",
            Self::Field => "field",
            Self::HoverCard => "hover-card",
            Self::InputGroup => "input-group",
            Self::InputOtp => "input-otp",
            Self::Item => "item",
            Self::Marker => "marker",
            Self::Message => "message",
            Self::MessageScroller => "message-scroller",
            Self::NativeSelect => "native-select",
            Self::NavigationMenu => "navigation-menu",
            Self::Questionnaire => "questionnaire",
            Self::ScrollArea => "scroll-area",
            Self::Sheet => "sheet",
            Self::Sidebar => "sidebar",
            Self::Toggle => "toggle",
            Self::ToggleGroup => "toggle-group",

            Self::Button => "button",
            Self::Badge => "badge",
            Self::Card => "card",
            Self::Alert => "alert",
            Self::Separator => "separator",
            Self::Collapsible => "collapsible",
            Self::Avatar => "avatar",
            Self::Empty => "empty",
            Self::FilePicker => "file-picker",
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
            Self::AnimatedText => "animated-text",
            Self::ColorPicker => "color-picker",
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
            Self::LiquidGlass => "liquid-glass",
            Self::ScrollShadow => "scroll-shadow",
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
            Self::Accordion => "Expand related sections with the keyboard.",
            Self::AlertDialog => "Confirm an action with a safe initial focus.",
            Self::Attachment => "File previews and transfer states.",
            Self::Bubble => "Conversation surfaces and reactions.",
            Self::ButtonGroup => "Related actions in a named group.",
            Self::Carousel => "Browse slides with buttons, arrows or a swipe.",
            Self::Chart => "Explore series with a zero baseline and accessible data points.",
            Self::Combobox => "Filter choices while keeping focus in the search field.",
            Self::Direction => "Share left-to-right or right-to-left layout with nested content.",
            Self::Drawer => "Drag the handle or use Close to dismiss the panel.",
            Self::Field => "Labels, help and errors linked to their controls.",
            Self::HoverCard => "A rich preview on hover or keyboard focus.",
            Self::InputGroup => "Inputs with attached labels and actions.",
            Self::InputOtp => "Enter or paste a numeric verification code.",
            Self::Item => "Reusable rows with media, details and actions.",
            Self::Marker => "Notes and status markers within a conversation.",
            Self::Message => "Messages with authors, metadata and actions.",
            Self::MessageScroller => "Keep your place while new messages arrive.",
            Self::NativeSelect => "A compact form select drawn by Argui.",
            Self::NavigationMenu => "Navigate links and explore anchored panels.",
            Self::Questionnaire => "Answer, skip and submit a sequence of questions.",
            Self::ScrollArea => "A keyboard-accessible viewport with themed scrollbars.",
            Self::Sheet => "Modal panels attached to an edge of the window.",
            Self::Sidebar => "Collapsible navigation with a compact rail.",
            Self::Toggle => "Press to switch a persistent formatting option.",
            Self::ToggleGroup => "Select one or several options with roving keyboard focus.",

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
            Self::FilePicker => {
                "Choose files, folders or a save destination in your system’s native dialog."
            }
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
            Self::AnimatedText => {
                "Rolling digits, sliding labels and quiet fades; unchanged characters stay still."
            }
            Self::ColorPicker => {
                "Precise color editing with opacity, a saturation pad and HEX/RGB/HSL/HSV fields."
            }
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
            Self::LiquidGlass => "Scroll colorful palettes behind a floating glass navigation bar.",
            Self::ScrollShadow => "Edge shadows reveal more content in scrollable views.",
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
