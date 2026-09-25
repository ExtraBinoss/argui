use std::collections::HashSet;

use argui_core::Name;

use super::{ShaderDiagnostic, ShaderError, ValidatedShader, wrap_effect_source};

/// Shader-visible parameter metadata checked independently from the renderer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderParameterMetadata {
    pub name: Name,
    pub words: usize,
}

impl ShaderParameterMetadata {
    /// Creates shader parameter metadata.
    ///
    /// * `name` — unique public parameter name.
    /// * `words` — packed 32-bit word count, which must be non-zero.
    #[must_use]
    pub fn new(name: impl Into<Name>, words: usize) -> Self {
        Self {
            name: name.into(),
            words,
        }
    }
}

/// Validates parameter metadata, ABI wrapping, WGSL syntax, and Naga semantics.
///
/// * `source_name` — developer-facing path used in diagnostics.
/// * `user_source` — custom effect WGSL.
/// * `parameters` — ordered packed parameter metadata.
///
/// # Errors
///
/// Returns a source-mapped diagnostic or invalid parameter metadata.
pub fn validate_effect_source(
    source_name: impl Into<String>,
    user_source: &str,
    parameters: &[ShaderParameterMetadata],
) -> Result<ValidatedShader, ShaderError> {
    validate_parameters(parameters)?;
    let wrapped = wrap_effect_source(source_name, user_source);
    let module = naga::front::wgsl::parse_str(&wrapped.source).map_err(|error| {
        let (span, label) = error
            .labels()
            .next()
            .unwrap_or((naga::Span::UNDEFINED, error.message()));
        let position = wrapped
            .source_map
            .map_span(span, user_source)
            .unwrap_or_else(|| wrapped.source_map.generated_position(span, &wrapped.source));
        ShaderError::Diagnostic(ShaderDiagnostic {
            position,
            message: format!("{}: {label}", error.message()),
        })
    })?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| {
        let span = error
            .spans()
            .next()
            .map_or(naga::Span::UNDEFINED, |(span, _)| *span);
        let position = wrapped
            .source_map
            .map_span(span, user_source)
            .unwrap_or_else(|| wrapped.source_map.generated_position(span, &wrapped.source));
        ShaderError::Diagnostic(ShaderDiagnostic {
            position,
            message: error.to_string(),
        })
    })?;
    Ok(ValidatedShader {
        source: wrapped.source,
        source_map: wrapped.source_map,
        hash: wrapped.hash,
    })
}

fn validate_parameters(parameters: &[ShaderParameterMetadata]) -> Result<(), ShaderError> {
    let mut names = HashSet::with_capacity(parameters.len());
    for parameter in parameters {
        if parameter.name.as_str().is_empty() {
            return Err(ShaderError::InvalidParameter(
                "parameter names cannot be empty".into(),
            ));
        }
        if parameter.words == 0 {
            return Err(ShaderError::InvalidParameter(format!(
                "parameter `{}` occupies zero words",
                parameter.name
            )));
        }
        if !names.insert(parameter.name.clone()) {
            return Err(ShaderError::InvalidParameter(format!(
                "parameter `{}` is declared more than once",
                parameter.name
            )));
        }
    }
    Ok(())
}
