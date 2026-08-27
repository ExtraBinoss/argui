use argui_paint::{VectorAsset, VectorId};
use argui_vector::{morph_svg, parse_svg};
use icondata_core::IconData;
use icondata_tb::{TbChevronDownOutline, TbChevronRightOutline, TbCopyOutline, TbTargetOutline};

#[derive(Debug)]
pub(crate) struct DevtoolsIcons {
    pub(crate) chevron: VectorId,
    pub(crate) copy: VectorId,
    pub(crate) target: VectorId,
    assets: Vec<VectorAsset>,
}

impl DevtoolsIcons {
    pub(crate) fn embedded() -> Self {
        let chevron = VectorId::fresh();
        let copy = VectorId::fresh();
        let target = VectorId::fresh();
        let right = tabler_svg(TbChevronRightOutline);
        let down = tabler_svg(TbChevronDownOutline);
        let assets = vec![
            morph_svg(chevron, right.as_bytes(), down.as_bytes())
                .expect("official Tabler chevrons form a compatible GPU morph"),
            tabler(copy, TbCopyOutline),
            tabler(target, TbTargetOutline),
        ];
        Self {
            chevron,
            copy,
            target,
            assets,
        }
    }

    pub(crate) fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }
}

fn tabler(id: VectorId, icon: &IconData) -> VectorAsset {
    let svg = tabler_svg(icon);
    parse_svg(id, svg.as_bytes()).expect("official Tabler icon data is valid SVG")
}

fn tabler_svg(icon: &IconData) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{}" fill="{}" stroke="#d9e3f2" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}">{}</svg>"##,
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
