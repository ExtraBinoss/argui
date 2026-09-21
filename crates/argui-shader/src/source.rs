use naga::Span;

use crate::ShaderSourcePosition;

const ABI_HEADER: &str = include_str!("custom_abi_header.wgsl");
const ABI_FOOTER: &str = include_str!("custom_abi_footer.wgsl");

/// Maps offsets in ABI-wrapped WGSL back to the original user source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderSourceMap {
    source_name: String,
    prefix_bytes: usize,
    user_bytes: usize,
}

impl ShaderSourceMap {
    /// Returns the developer-visible source name.
    #[must_use]
    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    /// Maps a Naga span to a user-source position when the span begins inside user WGSL.
    ///
    /// * `span` — span in the generated ABI-wrapped source.
    /// * `user_source` — exact user source used to create this map.
    #[must_use]
    pub fn map_span(&self, span: Span, user_source: &str) -> Option<ShaderSourcePosition> {
        let range = span.to_range()?;
        if range.start < self.prefix_bytes
            || range.start >= self.prefix_bytes.saturating_add(self.user_bytes)
        {
            return None;
        }
        let offset = range.start - self.prefix_bytes;
        let prefix = user_source.get(..offset)?;
        let line = prefix.matches('\n').count() as u32 + 1;
        let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
        Some(ShaderSourcePosition {
            source_name: self.source_name.clone(),
            line,
            column: (offset - line_start) as u32 + 1,
        })
    }

    pub(crate) fn generated_position(
        &self,
        span: Span,
        wrapped_source: &str,
    ) -> ShaderSourcePosition {
        let location = span.location(wrapped_source);
        ShaderSourcePosition {
            source_name: "<argui-effect-abi>".into(),
            line: location.line_number,
            column: location.line_position,
        }
    }
}

/// Custom-effect source after adding the Argui ABI declarations and entry points.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrappedShader {
    pub source: String,
    pub source_map: ShaderSourceMap,
    pub hash: u64,
}

/// Successfully parsed and validated custom-effect source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedShader {
    pub source: String,
    pub source_map: ShaderSourceMap,
    pub hash: u64,
}

/// Wraps user WGSL in the stable Argui custom-effect ABI and computes a deterministic hash.
///
/// * `source_name` — developer-facing path used by diagnostics.
/// * `user_source` — custom `argui_effect` implementation.
#[must_use]
pub fn wrap_effect_source(source_name: impl Into<String>, user_source: &str) -> WrappedShader {
    let prefix = format!("{ABI_HEADER}\n");
    let source = format!("{prefix}{user_source}\n{ABI_FOOTER}");
    WrappedShader {
        hash: fnv1a64(source.as_bytes()),
        source_map: ShaderSourceMap {
            source_name: source_name.into(),
            prefix_bytes: prefix.len(),
            user_bytes: user_source.len(),
        },
        source,
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
