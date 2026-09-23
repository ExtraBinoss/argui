use argui_dsl_ir::{AssetId, IrProject};
use serde::{Deserialize, Serialize};

/// Current framing and message schema version.
pub const PROTOCOL_VERSION: u16 = 2;
/// Current serialized typed-IR schema version.
pub const IR_FORMAT_VERSION: u16 = 11;
/// Engine ABI version accepted by this protocol build.
pub const ENGINE_COMPATIBILITY_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compatibility and transactional identity preceding every live package.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageHeader {
    pub protocol_version: u16,
    pub ir_format_version: u16,
    pub engine_version: String,
    pub public_api_hash: u64,
    pub generation: u64,
}

impl PackageHeader {
    /// Creates a header for a newly compiled generation.
    ///
    /// * `public_api_hash` — deterministic Rust-facing root ABI hash.
    /// * `generation` — monotonically increasing compiler generation.
    #[must_use]
    pub fn current(public_api_hash: u64, generation: u64) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            ir_format_version: IR_FORMAT_VERSION,
            engine_version: ENGINE_COMPATIBILITY_VERSION.into(),
            public_api_hash,
            generation: generation.max(1),
        }
    }
}

/// Complete bytes for one stable asset identity and source revision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetBytes {
    pub id: AssetId,
    pub revision: u64,
    pub bytes: Vec<u8>,
}

/// All data required to prepare one live generation without a compiler in-app.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LivePackageEnvelope {
    pub header: PackageHeader,
    pub roots: Vec<argui_dsl_ir::ComponentId>,
    pub ir: IrProject,
    pub assets: Vec<AssetBytes>,
}

/// Stable diagnostic severity transported from the host compiler.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// Source-located compiler or reload diagnostic safe for every transport.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticMessage {
    pub path: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub severity: Severity,
    pub code: String,
    pub message: String,
}

/// Bidirectional messages shared by local TCP and WebSocket transports.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LiveMessage {
    Hello {
        protocol_version: u16,
        ir_format_version: u16,
        engine_version: String,
    },
    Package(Box<LivePackageEnvelope>),
    Diagnostics {
        generation: u64,
        diagnostics: Vec<DiagnosticMessage>,
    },
    Committed {
        generation: u64,
    },
    Rejected {
        generation: u64,
        message: String,
    },
    RestartRequired {
        generation: u64,
        previous_api_hash: u64,
        next_api_hash: u64,
    },
}
