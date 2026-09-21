//! Lossless concrete syntax types and typed AST views for the Argui DSL.

pub mod ast;
mod kind;
mod text;
pub mod tree;

pub use kind::SyntaxKind;
pub use rowan::{GreenNode, TextRange, TextSize};
pub use text::{FileId, Span};
pub use tree::{ArguiLanguage, SyntaxElement, SyntaxNode, SyntaxToken};
