//! Evaluation of gradient constructors in live bytecode.

use argui_dsl_ir::BuiltinFunction;

use super::type_error;
use crate::{DslValue, RuntimeError};

/// Builds a GPU fill from typed live values and reports invalid authored stops.
///
/// * `function` — linear, radial, or conic gradient constructor.
/// * `arguments` — color array, offset array, geometry, and color space.
///
/// # Errors
///
/// Returns a bytecode error when the values or gradient stops are invalid.
pub(super) fn gradient_value(
    function: BuiltinFunction,
    arguments: &[DslValue],
) -> Result<DslValue, RuntimeError> {
    let colors = match arguments.first() {
        Some(DslValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                DslValue::Color(color) => Ok(*color),
                other => Err(type_error("color", other)),
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => return Err(type_error("array<color>", other)),
        None => {
            return Err(RuntimeError::InvalidBytecode(
                "gradient colors are missing".into(),
            ));
        }
    };
    let offsets = match arguments.get(1) {
        Some(DslValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                DslValue::Float(number) => Ok(*number as f32),
                DslValue::Int(number) => Ok(*number as f32),
                other => Err(type_error("float", other)),
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => return Err(type_error("array<float>", other)),
        None => {
            return Err(RuntimeError::InvalidBytecode(
                "gradient offsets are missing".into(),
            ));
        }
    };
    let number = |index| match arguments.get(index) {
        Some(DslValue::Float(value)) => Ok(*value as f32),
        Some(DslValue::Int(value)) => Ok(*value as f32),
        Some(other) => Err(type_error("float", other)),
        None => Err(RuntimeError::InvalidBytecode(format!(
            "gradient geometry argument {index} is missing"
        ))),
    };
    let space_index = match function {
        BuiltinFunction::LinearGradient => 3,
        BuiltinFunction::RadialGradient => 6,
        BuiltinFunction::ConicGradient => 5,
        _ => {
            return Err(RuntimeError::InvalidBytecode(
                "not a gradient function".into(),
            ));
        }
    };
    let space = match arguments.get(space_index) {
        Some(DslValue::String(value)) => value.as_str(),
        Some(other) => return Err(type_error("string", other)),
        None => {
            return Err(RuntimeError::InvalidBytecode(
                "gradient color space is missing".into(),
            ));
        }
    };
    let fill = match function {
        BuiltinFunction::LinearGradient => {
            argui_paint::Fill::linear_gradient(&colors, &offsets, number(2)?, space)
        }
        BuiltinFunction::RadialGradient => argui_paint::Fill::radial_gradient(
            &colors,
            &offsets,
            argui_core::Point::new(number(2)?, number(3)?),
            argui_core::Point::new(number(4)?, number(5)?),
            space,
        ),
        BuiltinFunction::ConicGradient => argui_paint::Fill::conic_gradient(
            &colors,
            &offsets,
            argui_core::Point::new(number(2)?, number(3)?),
            number(4)?,
            space,
        ),
        _ => unreachable!("the gradient kind was already checked"),
    }
    .map_err(|error| RuntimeError::InvalidBytecode(format!("invalid gradient: {error}")))?;
    Ok(DslValue::Brush(fill))
}
