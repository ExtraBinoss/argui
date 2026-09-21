//! Lossless, error-recovering lexer and parser for the Argui DSL.

mod error;
mod format;
mod grammar;
pub mod lexer;
mod parser;

use argui_dsl_syntax::{SyntaxNode, ast};

pub use error::ParseDiagnostic;
pub use format::format_source;

/// Lossless parse result containing a CST even when diagnostics are present.
#[derive(Clone, Debug)]
pub struct Parse {
    green: rowan::GreenNode,
    diagnostics: Vec<ParseDiagnostic>,
}

impl Parse {
    /// Returns the lossless concrete syntax root.
    #[must_use]
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Returns the immutable, thread-safe green tree backing this parse.
    #[must_use]
    pub fn green(&self) -> argui_dsl_syntax::GreenNode {
        self.green.clone()
    }

    /// Returns the typed source-file AST facade.
    #[must_use]
    pub fn tree(&self) -> ast::SourceFile {
        use ast::AstNode;
        ast::SourceFile::cast(self.syntax()).expect("parser always creates a root node")
    }

    /// Returns syntax and recovery diagnostics in source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }

    /// Returns whether parsing produced no diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Parses Argui source into a lossless CST and typed AST facade.
///
/// Invalid and incomplete input still produces a tree containing every source byte.
///
/// * `source` — complete or partially edited UTF-8 source text.
#[must_use]
pub fn parse(source: &str) -> Parse {
    parser::Parser::new(source).parse()
}
