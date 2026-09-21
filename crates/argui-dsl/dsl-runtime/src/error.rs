/// Deterministic live-package preparation, migration, or evaluation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    MissingComponent(u64),
    MissingProperty(u64),
    MissingExpression(u64),
    MissingAsset(u64),
    TypeMismatch { expected: String, actual: String },
    InvalidBytecode(String),
    InvalidShader(String),
    IncompatiblePackage(String),
    Asset(String),
    Schema(String),
    RestartRequired { previous: u64, next: u64 },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingComponent(id) => write!(formatter, "component {id} is unavailable"),
            Self::MissingProperty(id) => write!(formatter, "property {id} is unavailable"),
            Self::MissingExpression(id) => write!(formatter, "expression {id} is unavailable"),
            Self::MissingAsset(id) => write!(formatter, "asset {id} is unavailable"),
            Self::TypeMismatch { expected, actual } => {
                write!(formatter, "expected {expected}, found {actual}")
            }
            Self::InvalidBytecode(message) => write!(formatter, "invalid bytecode: {message}"),
            Self::InvalidShader(message) => write!(formatter, "invalid shader: {message}"),
            Self::IncompatiblePackage(message) => {
                write!(formatter, "incompatible live package: {message}")
            }
            Self::Asset(message) => write!(formatter, "asset: {message}"),
            Self::Schema(message) => write!(formatter, "native schema: {message}"),
            Self::RestartRequired { previous, next } => write!(
                formatter,
                "public UI ABI changed from {previous:016x} to {next:016x}; restart required"
            ),
        }
    }
}

impl std::error::Error for RuntimeError {}
