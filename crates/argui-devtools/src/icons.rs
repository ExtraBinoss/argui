use argui_paint::{VectorAsset, VectorId};
use argui_vector::parse_svg;
use icondata_core::IconData;
use icondata_tb::{
    TbBoxOutline, TbChevronRightOutline, TbCopyOutline, TbFileTextOutline, TbPhotoOutline,
    TbPointerOutline, TbTargetOutline, TbVectorOutline,
};

#[derive(Debug)]
pub(crate) struct DevtoolsIcons {
    pub(crate) chevron: VectorId,
    pub(crate) copy: VectorId,
    pub(crate) target: VectorId,
    node_icons: [VectorId; 5],
    assets: Vec<VectorAsset>,
}

impl DevtoolsIcons {
    pub(crate) fn embedded() -> Self {
        let chevron = VectorId::fresh();
        let copy = VectorId::fresh();
        let target = VectorId::fresh();
        let mut assets = vec![
            tabler(chevron, TbChevronRightOutline),
            tabler(copy, TbCopyOutline),
            tabler(target, TbTargetOutline),
        ];
        let node_icons = std::array::from_fn(|_| VectorId::fresh());
        for (id, icon) in node_icons.into_iter().zip([
            TbBoxOutline,
            TbFileTextOutline,
            TbPhotoOutline,
            TbVectorOutline,
            TbPointerOutline,
        ]) {
            assets.push(tabler(id, icon));
        }
        Self {
            chevron,
            copy,
            target,
            node_icons,
            assets,
        }
    }

    pub(crate) fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }

    pub(crate) fn node_icon(&self, kind: &str) -> VectorId {
        self.node_icons[match kind {
            "text" => 1,
            "image" => 2,
            "vector" => 3,
            "text-input" | "text-area" => 4,
            _ => 0,
        }]
    }
}

fn tabler(id: VectorId, icon: &IconData) -> VectorAsset {
    let svg = tabler_svg(icon);
    parse_svg(id, svg.as_bytes()).expect("official Tabler icon data is valid SVG")
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
