//! Incremental, backend-independent semantic analysis for Argui DSL projects.

mod check;
mod db;
mod diagnostic;
mod lower;
mod model;
mod path;
mod tooling;
mod types;

pub use db::{CompilerDatabase, QueryStats};
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use model::{
    CallbackDefinition, ComponentDefinition, Definition, DefinitionKind, EffectDefinition,
    EffectParameterDefinition, EnumDefinition, FieldDefinition, Import, ImportItem, Module,
    PropertyDefinition, PropertyDirection, SemanticProject, SlotDefinition, StructDefinition,
    StyleDefinition, SymbolId, ThemeDefinition, ThemeTokenDefinition,
};
pub use tooling::{
    CodeAction, ColorPresentation, Completion, Location, RenameEdit, SemanticClass,
    SemanticHighlight, Symbol, SymbolKind, ToolingError,
};
pub use types::Type;
