//! WGSL validation and source mapped diagnostics for custom effects.

mod diagnostic;
mod source;
mod validate;

pub use diagnostic::{ShaderDiagnostic, ShaderError, ShaderSourcePosition};
pub use source::{ShaderSourceMap, ValidatedShader, WrappedShader, wrap_effect_source};
pub use validate::{ShaderParameterMetadata, validate_effect_source};
