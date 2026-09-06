#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Page {
    #[default]
    Button,
    Input,
    TextArea,
    Checkbox,
    Switch,
    RadioGroup,
    Slider,
    Tabs,
    Select,
    Dialog,
    Form,
    Settings,
    Layout,
    Motion,
    Effects,
    Composition,
    Typography,
    WebView,
}

impl Page {
    pub const ALL: [Self; 18] = [
        Self::Button,
        Self::Input,
        Self::TextArea,
        Self::Checkbox,
        Self::Switch,
        Self::RadioGroup,
        Self::Slider,
        Self::Tabs,
        Self::Select,
        Self::Dialog,
        Self::Form,
        Self::Settings,
        Self::Layout,
        Self::Motion,
        Self::Effects,
        Self::Composition,
        Self::Typography,
        Self::WebView,
    ];

    pub const fn category(self) -> &'static str {
        match self {
            Self::Button
            | Self::Input
            | Self::TextArea
            | Self::Checkbox
            | Self::Switch
            | Self::RadioGroup
            | Self::Slider
            | Self::Tabs
            | Self::Select
            | Self::Dialog => "Widgets",
            Self::Form
            | Self::Settings
            | Self::Layout
            | Self::Motion
            | Self::Effects
            | Self::Composition
            | Self::Typography
            | Self::WebView => "Examples",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Input => "Input & Search",
            Self::TextArea => "Text area",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::RadioGroup => "Radio group",
            Self::Slider => "Slider",
            Self::Tabs => "Tabs",
            Self::Select => "Select",
            Self::Dialog => "Dialog",
            Self::Form => "Profile form",
            Self::Settings => "Settings panel",
            Self::Layout => "Web layout",
            Self::Motion => "Motion & loading",
            Self::Effects => "GPU effects / WGSL",
            Self::Composition => "Advanced composition",
            Self::Typography => "Typography & selection",
            Self::WebView => "WebView",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Input => "input",
            Self::TextArea => "textarea",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::RadioGroup => "radio-group",
            Self::Slider => "slider",
            Self::Tabs => "tabs",
            Self::Select => "select",
            Self::Dialog => "dialog",
            Self::Form => "form",
            Self::Settings => "settings",
            Self::Layout => "layout",
            Self::Motion => "motion",
            Self::Effects => "effects",
            Self::Composition => "composition",
            Self::Typography => "typography",
            Self::WebView => "webview",
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
