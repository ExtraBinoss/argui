//! Versioned, length-framed transport model for validated Argui DSL generations.

mod codec;
mod message;
mod versions;

pub use codec::{MAX_FRAME_BYTES, decode, encode, read_frame, write_frame};
pub use message::{
    AssetBytes, DiagnosticMessage, ENGINE_COMPATIBILITY_VERSION, IR_FORMAT_VERSION, LiveMessage,
    LivePackageEnvelope, PROTOCOL_VERSION, PackageHeader, Severity,
};
pub use versions::{CompatibilityError, RuntimeVersions};
