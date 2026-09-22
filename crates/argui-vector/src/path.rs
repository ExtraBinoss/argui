use core::fmt::Write;

use argui_core::{Point, Size};
use argui_paint::{VectorAsset, VectorId};

use crate::{VectorError, parse_svg};

/// An absolute command in a vector path's local coordinate system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadraticTo {
        control: Point,
        end: Point,
    },
    CubicTo {
        first_control: Point,
        second_control: Point,
        end: Point,
    },
    Close,
}

/// Paint applied to a path using the color of its [`argui_paint::VectorPrimitive`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathStyle {
    /// Whether to fill the path's enclosed area.
    pub fill: bool,
    /// Optional stroke width in local path coordinates.
    pub stroke_width: Option<f32>,
    /// Whether enclosed areas use the even-odd rather than nonzero fill rule.
    pub even_odd: bool,
}

impl PathStyle {
    /// Creates a filled path with no stroke.
    #[must_use]
    pub const fn filled() -> Self {
        Self {
            fill: true,
            stroke_width: None,
            even_odd: false,
        }
    }

    /// Creates an unfilled path with a stroke of `width` local units.
    ///
    /// Invalid widths are rejected by [`path_asset`].
    #[must_use]
    pub const fn stroked(width: f32) -> Self {
        Self {
            fill: false,
            stroke_width: Some(width),
            even_odd: false,
        }
    }
}

/// Validation or SVG parsing error while constructing a path asset.
#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("path view size must be finite and positive")]
    InvalidSize,
    #[error("path must contain at least one drawing command")]
    Empty,
    #[error("path command {0} must follow a move command in an open subpath")]
    InvalidCommand(usize),
    #[error("path command {0} contains a non-finite coordinate")]
    InvalidCoordinate(usize),
    #[error("path stroke width must be finite and positive")]
    InvalidStroke,
    #[error("path style must have a fill or stroke")]
    EmptyStyle,
    #[error(transparent)]
    Svg(#[from] VectorError),
}

/// Builds a resolution-independent path asset from absolute geometry.
///
/// `id` identifies the resulting asset, `size` defines its local view box,
/// `commands` describes one or more subpaths, and `style` selects fill and
/// stroke paint. The resulting asset can be drawn at any bounds with a
/// [`argui_paint::VectorPrimitive`]; its color, opacity, transform, and clips
/// remain per-instance properties.
///
/// # Errors
/// Returns [`PathError`] for invalid dimensions, coordinates, command order,
/// or paint, or if the generated SVG cannot be parsed.
pub fn path_asset(
    id: VectorId,
    size: Size,
    commands: &[PathCommand],
    style: PathStyle,
) -> Result<VectorAsset, PathError> {
    if !size.width.is_finite()
        || !size.height.is_finite()
        || size.width <= 0.0
        || size.height <= 0.0
    {
        return Err(PathError::InvalidSize);
    }
    if let Some(width) = style.stroke_width
        && (!width.is_finite() || width <= 0.0)
    {
        return Err(PathError::InvalidStroke);
    }
    if !style.fill && style.stroke_width.is_none() {
        return Err(PathError::EmptyStyle);
    }

    let mut data = String::new();
    let mut open = false;
    let mut drawn = false;
    for (index, command) in commands.iter().enumerate() {
        match *command {
            PathCommand::MoveTo(point) => {
                check_points(index, &[point])?;
                write!(data, "M{} {}", point.x, point.y).expect("writing to a string cannot fail");
                open = true;
            }
            PathCommand::LineTo(point) if open => {
                check_points(index, &[point])?;
                write!(data, "L{} {}", point.x, point.y).expect("writing to a string cannot fail");
                drawn = true;
            }
            PathCommand::QuadraticTo { control, end } if open => {
                check_points(index, &[control, end])?;
                write!(data, "Q{} {} {} {}", control.x, control.y, end.x, end.y)
                    .expect("writing to a string cannot fail");
                drawn = true;
            }
            PathCommand::CubicTo {
                first_control,
                second_control,
                end,
            } if open => {
                check_points(index, &[first_control, second_control, end])?;
                write!(
                    data,
                    "C{} {} {} {} {} {}",
                    first_control.x,
                    first_control.y,
                    second_control.x,
                    second_control.y,
                    end.x,
                    end.y
                )
                .expect("writing to a string cannot fail");
                drawn = true;
            }
            PathCommand::Close if open => {
                data.push('Z');
                open = false;
            }
            _ => return Err(PathError::InvalidCommand(index)),
        }
    }
    if !drawn {
        return Err(PathError::Empty);
    }

    let fill = if style.fill { "currentColor" } else { "none" };
    let stroke = if style.stroke_width.is_some() {
        "currentColor"
    } else {
        "none"
    };
    let width = style.stroke_width.unwrap_or(0.0);
    let rule = if style.even_odd { "evenodd" } else { "nonzero" };
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><path d=\"{}\" fill=\"{}\" fill-rule=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/></svg>",
        size.width, size.height, size.width, size.height, data, fill, rule, stroke, width
    );
    Ok(parse_svg(id, svg.as_bytes())?)
}

/// Rejects non-finite points before their coordinates are written to SVG.
fn check_points(index: usize, points: &[Point]) -> Result<(), PathError> {
    if points
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite())
    {
        Ok(())
    } else {
        Err(PathError::InvalidCoordinate(index))
    }
}
