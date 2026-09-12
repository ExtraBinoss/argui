use argui_core::Size;
use argui_paint::{VectorAsset, VectorId};

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("SVG parsing failed: {0}")]
    Svg(#[from] usvg::Error),
}

pub fn parse_svg(id: VectorId, svg: &[u8]) -> Result<VectorAsset, VectorError> {
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default())?;
    let width = tree.size().width();
    let height = tree.size().height();
    Ok(VectorAsset {
        id,
        size: Size::new(width, height),
        svg: svg.into(),
        tintable: contains_current_color(svg),
    })
}

fn contains_current_color(svg: &[u8]) -> bool {
    svg.windows(b"currentColor".len())
        .any(|window| window.eq_ignore_ascii_case(b"currentColor"))
}
