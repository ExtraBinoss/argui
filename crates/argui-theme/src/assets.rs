use argui_paint::{Color, VectorAsset, VectorId};
use argui_ui::{Element, LayoutStyle, Length};
use argui_vector::parse_svg;
use icondata_core::IconData;
use icondata_tb::TbResizeOutline;

#[derive(Clone, Debug)]
pub struct WidgetAssets {
    resize_handle: VectorId,
    assets: Vec<VectorAsset>,
}

impl WidgetAssets {
    #[must_use]
    pub fn embedded(color: Color) -> Self {
        let resize_handle = VectorId::fresh();
        let svg = tabler_svg(TbResizeOutline, color);
        let asset = parse_svg(resize_handle, svg.as_bytes())
            .expect("the embedded Tabler resize icon is valid SVG");
        Self {
            resize_handle,
            assets: vec![asset],
        }
    }

    #[must_use]
    pub fn resize_handle(&self) -> Element {
        Element::vector(self.resize_handle).layout_style(LayoutStyle {
            width: Length::Px(18.0),
            height: Length::Px(18.0),
            ..LayoutStyle::default()
        })
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
