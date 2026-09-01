use argui_core::Size;
use argui_paint::{VectorAsset, VectorId};

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("SVG parsing failed: {0}")]
    Svg(#[from] usvg::Error),
    #[error("SVG has no drawable area")]
    Empty,
}

pub fn parse_svg(id: VectorId, svg: &[u8]) -> Result<VectorAsset, VectorError> {
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default())?;
    let width = tree.size().width();
    let height = tree.size().height();
    if width <= 0.0 || height <= 0.0 {
        return Err(VectorError::Empty);
    }
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

#[cfg(test)]
mod tests {
    use super::contains_current_color;

    #[test]
    fn detects_current_color_case_insensitively() {
        assert!(contains_current_color(b"stroke='currentColor'"));
        assert!(contains_current_color(b"fill='CURRENTCOLOR'"));
        assert!(!contains_current_color(b"stroke='#fff'"));
    }
}
