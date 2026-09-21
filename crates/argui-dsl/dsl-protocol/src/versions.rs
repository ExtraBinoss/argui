use crate::{ENGINE_COMPATIBILITY_VERSION, IR_FORMAT_VERSION, PROTOCOL_VERSION, PackageHeader};

/// Exact protocol versions supported by a running development application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeVersions {
    pub protocol: u16,
    pub ir: u16,
    pub engine: String,
}

impl RuntimeVersions {
    /// Returns the versions compiled into the current runtime.
    #[must_use]
    pub fn current() -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            ir: IR_FORMAT_VERSION,
            engine: ENGINE_COMPATIBILITY_VERSION.into(),
        }
    }

    /// Rejects every unknown protocol, IR, or engine ABI version explicitly.
    ///
    /// # Errors
    ///
    /// Returns the exact mismatched version before package preparation starts.
    pub fn check(&self, header: &PackageHeader) -> Result<(), CompatibilityError> {
        if header.protocol_version != self.protocol {
            return Err(CompatibilityError::Protocol {
                expected: self.protocol,
                actual: header.protocol_version,
            });
        }
        if header.ir_format_version != self.ir {
            return Err(CompatibilityError::Ir {
                expected: self.ir,
                actual: header.ir_format_version,
            });
        }
        if header.engine_version != self.engine {
            return Err(CompatibilityError::Engine {
                expected: self.engine.clone(),
                actual: header.engine_version.clone(),
            });
        }
        Ok(())
    }
}

/// Incompatible compiler/runtime boundary detected before decoding live state.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CompatibilityError {
    #[error("protocol version {actual} is incompatible; runtime requires {expected}")]
    Protocol { expected: u16, actual: u16 },
    #[error("IR format version {actual} is incompatible; runtime requires {expected}")]
    Ir { expected: u16, actual: u16 },
    #[error("engine version {actual} is incompatible; runtime requires {expected}")]
    Engine { expected: String, actual: String },
}
