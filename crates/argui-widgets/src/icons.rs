use argui_paint::{Color, VectorAsset, VectorId};
use argui_ui::{Dimensions, Element, LayoutStyle, Semantics};
use argui_vector::parse_svg;
use icondata_core::IconData;
use icondata_tb::{
    TbAlertCircleOutline, TbAlertTriangleOutline, TbArrowDownOutline, TbArrowUpOutline,
    TbArrowsSortOutline, TbCalendarOutline, TbCheckOutline, TbChevronDownOutline,
    TbChevronLeftOutline, TbChevronRightOutline, TbCircleCheckOutline, TbCircleFilled,
    TbClipboardOutline, TbCopyOutline, TbCutOutline, TbDeviceDesktopOutline, TbInfoCircleOutline,
    TbLayoutSidebarLeftCollapseOutline, TbLoader2Outline, TbMinusOutline, TbMoonOutline,
    TbResizeOutline, TbRestoreOutline, TbSearchOutline, TbSelectAllOutline, TbSunOutline,
    TbXOutline,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum TablerIcon {
    Search,
    Sun,
    Moon,
    System,
    Check,
    ChevronDown,
    Close,
    Loader,
    Resize,
    Sidebar,
    Restore,
    Copy,
    Cut,
    Paste,
    SelectAll,
    ArrowUp,
    ArrowDown,
    ArrowsSort,
    ChevronLeft,
    ChevronRight,
    Calendar,
    Minus,
    Circle,
    Information,
    Success,
    Warning,
    Error,
}

impl TablerIcon {
    const ALL: [Self; 27] = [
        Self::Search,
        Self::Sun,
        Self::Moon,
        Self::System,
        Self::Check,
        Self::ChevronDown,
        Self::Close,
        Self::Loader,
        Self::Resize,
        Self::Sidebar,
        Self::Restore,
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::SelectAll,
        Self::ArrowUp,
        Self::ArrowDown,
        Self::ArrowsSort,
        Self::ChevronLeft,
        Self::ChevronRight,
        Self::Calendar,
        Self::Minus,
        Self::Circle,
        Self::Information,
        Self::Success,
        Self::Warning,
        Self::Error,
    ];

    const fn data(self) -> &'static IconData {
        match self {
            Self::Search => TbSearchOutline,
            Self::Sun => TbSunOutline,
            Self::Moon => TbMoonOutline,
            Self::System => TbDeviceDesktopOutline,
            Self::Check => TbCheckOutline,
            Self::ChevronDown => TbChevronDownOutline,
            Self::Close => TbXOutline,
            Self::Loader => TbLoader2Outline,
            Self::Resize => TbResizeOutline,
            Self::Sidebar => TbLayoutSidebarLeftCollapseOutline,
            Self::Restore => TbRestoreOutline,
            Self::Copy => TbCopyOutline,
            Self::Cut => TbCutOutline,
            Self::Paste => TbClipboardOutline,
            Self::SelectAll => TbSelectAllOutline,
            Self::ArrowUp => TbArrowUpOutline,
            Self::ArrowDown => TbArrowDownOutline,
            Self::ArrowsSort => TbArrowsSortOutline,

            Self::ChevronLeft => TbChevronLeftOutline,
            Self::ChevronRight => TbChevronRightOutline,
            Self::Calendar => TbCalendarOutline,
            Self::Minus => TbMinusOutline,
            Self::Circle => TbCircleFilled,
            Self::Information => TbInfoCircleOutline,
            Self::Success => TbCircleCheckOutline,
            Self::Warning => TbAlertTriangleOutline,
            Self::Error => TbAlertCircleOutline,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WidgetAssets {
    ids: [Option<VectorId>; 27],
    assets: Vec<VectorAsset>,
    color: Color,
}

impl WidgetAssets {
    /// Builds an asset bundle containing every embedded Tabler icon in `color`.
    #[must_use]
    pub fn tabler(color: Color) -> Self {
        Self::tabler_subset(color, TablerIcon::ALL)
    }

    #[must_use]
    /// Builds a themed bundle containing only the requested `TablerIcon` assets; `color` tints the generated icons.
    ///
    /// # Panics
    ///
    /// Panics if embedded Tabler icon data cannot be parsed as SVG.
    pub fn tabler_subset(color: Color, requested: impl IntoIterator<Item = TablerIcon>) -> Self {
        let mut ids = [None; 27];
        let icons = requested
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        let assets = TablerIcon::ALL
            .iter()
            .filter(|icon| icons.contains(icon))
            .map(|icon| {
                let id = VectorId::fresh();
                ids[*icon as usize] = Some(id);
                let svg = tabler_svg(icon.data());
                parse_svg(id, svg.as_bytes())
                    .expect("embedded Tabler icon data must remain valid SVG")
            })
            .collect();
        Self { ids, assets, color }
    }

    #[must_use]
    /// Creates a decorative icon element at `size` logical pixels.
    pub fn icon(&self, icon: TablerIcon, size: f32) -> Element {
        Element::vector(self.vector_id(icon))
            .layout_style(LayoutStyle {
                size: Dimensions::length(size),
                ..LayoutStyle::default()
            })
            .shrink(0.0)
            .vector_color(self.color)
            .semantic_hidden(true)
    }

    #[must_use]
    /// Returns the vector asset id for an icon included in this bundle.
    ///
    /// # Panics
    ///
    /// Panics if `icon` was not requested when this bundle was created.
    pub const fn vector_id(&self, icon: TablerIcon) -> VectorId {
        self.ids[icon as usize].expect("requested icon must be included in WidgetAssets")
    }

    #[must_use]
    /// Creates an icon with an accessible image label at `size` logical pixels.
    pub fn labeled_icon(&self, icon: TablerIcon, size: f32, label: impl Into<String>) -> Element {
        self.icon(icon, size)
            .semantic_hidden(false)
            .semantics(Semantics::new(argui_ui::Role::Image).label(label))
    }

    #[must_use]
    /// Returns the vector assets to register with the renderer.
    pub fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }
}

fn tabler_svg(icon: &IconData) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{}" fill="{}" stroke="currentColor" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}">{}</svg>"##,
        icon.width.unwrap_or("24"),
        icon.height.unwrap_or("24"),
        icon.view_box.unwrap_or("0 0 24 24"),
        icon.fill.unwrap_or("none"),
        icon.stroke_width.unwrap_or("2"),
        icon.stroke_linecap.unwrap_or("round"),
        icon.stroke_linejoin.unwrap_or("round"),
        icon.data,
    )
}
