use std::collections::HashMap;

use argui_dsl_ir::{ThemeModeId, TokenId};

use super::ValueContext;
use crate::{DslValue, LivePackage, RuntimeError};

/// Evaluates default theme tokens in dependency-checked declaration order.
///
/// * `package` — accepted package whose token expressions are precompiled.
///
/// Returns the complete default token map, including custom-theme replacements.
///
/// # Errors
///
/// Returns a bytecode or expression error when a token cannot be evaluated.
pub(super) fn evaluate_theme_defaults(
    package: &LivePackage,
) -> Result<HashMap<TokenId, DslValue>, RuntimeError> {
    let mut values = HashMap::new();
    for theme in &package.ir.themes {
        for token in &theme.tokens {
            let program = package
                .program(token.default.id)
                .ok_or(RuntimeError::MissingExpression(token.default.id.raw()))?;
            let properties = HashMap::new();
            let mut context = ValueContext::new(&properties, &values);
            values.insert(token.id, program.evaluate(&mut context)?);
        }
    }
    Ok(values)
}

/// Evaluates defaults and then overlays every theme sharing one named mode.
///
/// * `package` — accepted package containing the theme definitions.
/// * `mode` — stable identity of the selected mode spelling.
///
/// Returns the token map after applying all matching mode overrides.
///
/// # Errors
///
/// Returns an unknown-mode or expression error without changing runtime state.
pub(super) fn evaluate_theme_mode(
    package: &LivePackage,
    mode: ThemeModeId,
) -> Result<HashMap<TokenId, DslValue>, RuntimeError> {
    let mut values = evaluate_theme_defaults(package)?;
    let mut found = false;
    for selected in package
        .ir
        .themes
        .iter()
        .flat_map(|theme| &theme.modes)
        .filter(|candidate| candidate.id == mode)
    {
        found = true;
        for (token, expression) in &selected.overrides {
            let program = package
                .program(expression.id)
                .ok_or(RuntimeError::MissingExpression(expression.id.raw()))?;
            let properties = HashMap::new();
            let mut context = ValueContext::new(&properties, &values);
            values.insert(*token, program.evaluate(&mut context)?);
        }
    }
    if !found {
        return Err(RuntimeError::InvalidBytecode(format!(
            "unknown theme mode {}",
            mode.raw()
        )));
    }
    Ok(values)
}
