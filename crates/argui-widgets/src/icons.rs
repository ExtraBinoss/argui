use argui_paint::{Color, VectorAsset, VectorId};
use argui_ui::{Dimensions, Element, LayoutStyle, Semantics};
use argui_vector::parse_svg;
use icondata_core::IconData;
use icondata_tb::{
    TbCheckOutline, TbChevronDownOutline, TbDeviceDesktopOutline,
    TbLayoutSidebarLeftCollapseOutline, TbLoader2Outline, TbMoonOutline, TbResizeOutline,
    TbSearchOutline, TbSunOutline, TbXOutline,
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
}

impl TablerIcon {
    const ALL: [Self; 10] = [
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
        }
    }
}

#[derive(Clone, Debug)]
pub struct WidgetAssets {
    ids: [VectorId; 10],
    assets: Vec<VectorAsset>,
}

impl WidgetAssets {
    #[must_use]
    pub fn tabler(color: Color) -> Self {
        let ids = std::array::from_fn(|_| VectorId::fresh());
        let assets = TablerIcon::ALL
            .iter()
            .enumerate()
            .map(|(index, icon)| {
                let svg = tabler_svg(icon.data(), color);
                parse_svg(ids[index], svg.as_bytes())
                    .expect("embedded Tabler icon data must remain valid SVG")
            })
            .collect();
        Self { ids, assets }
    }

    #[must_use]
    pub fn icon(&self, icon: TablerIcon, size: f32) -> Element {
        Element::vector(self.ids[icon as usize])
            .layout_style(LayoutStyle {
                size: Dimensions::length(size),
                ..LayoutStyle::default()
            })
            .semantic_hidden(true)
    }

    #[must_use]
    pub const fn vector_id(&self, icon: TablerIcon) -> VectorId {
        self.ids[icon as usize]
    }

    #[must_use]
    pub fn labeled_icon(&self, icon: TablerIcon, size: f32, label: impl Into<String>) -> Element {
        self.icon(icon, size)
            .semantic_hidden(false)
            .semantics(Semantics::new(argui_ui::Role::Image).label(label))
    }

    #[must_use]
    pub fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }
}

fn tabler_svg(icon: &IconData, color: Color) -> String {
    let [red, green, blue, _] = color.as_array();
    let [red, green, blue] =
        [red, green, blue].map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{}" fill="{}" stroke="#{red:02x}{green:02x}{blue:02x}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}">{}</svg>"##,
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
