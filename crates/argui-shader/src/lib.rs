//! Renderer-independent validation for Argui custom-effect WGSL.

mod diagnostic;
mod source;
mod validate;

pub use diagnostic::{ShaderDiagnostic, ShaderError, ShaderSourcePosition};
pub use source::{ShaderSourceMap, ValidatedShader, WrappedShader, wrap_effect_source};
pub use validate::{ShaderParameterMetadata, validate_effect_source};
