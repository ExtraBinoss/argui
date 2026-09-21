mod completion;
mod navigation;
mod tokens;

use argui_dsl_syntax::{FileId, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TextSize};

use crate::{CompilerDatabase, Module, SemanticProject};

/// Editor completion candidate produced by the semantic database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completion {
    pub label: String,
    pub kind: SymbolKind,
    pub detail: String,
    pub documentation: String,
}

/// Source location expressed in canonical path and UTF-8 byte offsets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    pub path: String,
    pub start: u32,
    pub end: u32,
}

/// Workspace edit produced by a semantic rename operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenameEdit {
    pub location: Location,
    pub replacement: String,
}

/// Hierarchical source or workspace symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
}

/// Semantic symbol classification shared by LSP and JSON CLI output.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SymbolKind {
    Component,
    Property,
    Callback,
    Slot,
    Struct,
    Enum,
    Theme,
    Token,
    Style,
    Effect,
    Native,
    Keyword,
}

/// Semantic highlighting class for one source token.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticClass {
    Keyword,
    Type,
    Component,
    Property,
    Callback,
    Variable,
    String,
    Number,
    Comment,
    ThemeToken,
}

/// Byte-range semantic highlight emitted by the shared frontend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticHighlight {
    pub start: u32,
    pub end: u32,
    pub class: SemanticClass,
}

/// Safe source edit suggested for one common diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeAction {
    pub title: String,
    pub edit: RenameEdit,
}

/// Normalized color spelling for a source color literal.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorPresentation {
    pub label: String,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
    pub location: Location,
}

/// Invalid tooling request which cannot be associated with a project source.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ToolingError {
    #[error("module `{0}` is not loaded")]
    UnknownModule(String),
    #[error("byte offset {offset} is outside `{path}`")]
    InvalidOffset { path: String, offset: u32 },
    #[error("`{0}` is not a valid DSL identifier")]
    InvalidIdentifier(String),
}

/// Resolved source context shared by tooling queries.
struct SourceContext {
    project: std::sync::Arc<SemanticProject>,
    module: Module,
    file: FileId,
    root: SyntaxNode,
    token: Option<SyntaxToken>,
}

impl CompilerDatabase {
    /// Produces context-aware completion candidates at a UTF-8 byte offset.
    pub fn completions(
        &mut self,
        path: &str,
        offset: u32,
    ) -> Result<Vec<Completion>, ToolingError> {
        completion::completions(self, path, offset)
    }

    /// Returns Markdown hover text for the symbol at a UTF-8 byte offset.
    pub fn hover(&mut self, path: &str, offset: u32) -> Result<Option<String>, ToolingError> {
        navigation::hover(self, path, offset)
    }

    /// Resolves the declaration of the symbol at a UTF-8 byte offset.
    pub fn definition(
        &mut self,
        path: &str,
        offset: u32,
    ) -> Result<Option<Location>, ToolingError> {
        navigation::definition(self, path, offset)
    }

    /// Finds semantic references to the symbol at a UTF-8 byte offset.
    pub fn references(&mut self, path: &str, offset: u32) -> Result<Vec<Location>, ToolingError> {
        navigation::references(self, path, offset)
    }

    /// Computes workspace edits for a semantic symbol rename.
    pub fn rename(
        &mut self,
        path: &str,
        offset: u32,
        replacement: &str,
    ) -> Result<Vec<RenameEdit>, ToolingError> {
        navigation::rename(self, path, offset, replacement)
    }

    /// Returns document symbols or every user workspace symbol when `path` is absent.
    pub fn symbols(&mut self, path: Option<&str>) -> Result<Vec<Symbol>, ToolingError> {
        navigation::symbols(self, path)
    }

    /// Classifies source ranges for semantic highlighting.
    pub fn semantic_highlights(
        &mut self,
        path: &str,
    ) -> Result<Vec<SemanticHighlight>, ToolingError> {
        tokens::semantic_highlights(self, path)
    }

    /// Suggests safe single-token fixes for common semantic diagnostics.
    pub fn code_actions(&mut self, path: &str) -> Result<Vec<CodeAction>, ToolingError> {
        tokens::code_actions(self, path)
    }

    /// Returns normalized color data for the literal at a UTF-8 byte offset.
    pub fn color_presentation(
        &mut self,
        path: &str,
        offset: u32,
    ) -> Result<Option<ColorPresentation>, ToolingError> {
        tokens::color_presentation(self, path, offset)
    }

    /// Resolves one loaded file, checked module, CST root, and nearby token.
    fn source_context(&mut self, path: &str, offset: u32) -> Result<SourceContext, ToolingError> {
        let file = self
            .file_id(path)
            .ok_or_else(|| ToolingError::UnknownModule(path.into()))?;
        let source = self
            .source(file)
            .ok_or_else(|| ToolingError::UnknownModule(path.into()))?;
        if usize::try_from(offset).map_or(true, |offset| offset > source.len()) {
            return Err(ToolingError::InvalidOffset {
                path: path.into(),
                offset,
            });
        }
        let project = self.check();
        let module = project
            .modules
            .iter()
            .find(|module| module.file == file)
            .cloned()
            .ok_or_else(|| ToolingError::UnknownModule(path.into()))?;
        let root = project
            .syntax(file)
            .ok_or_else(|| ToolingError::UnknownModule(path.into()))?;
        let token = token_at(&root, offset);
        Ok(SourceContext {
            project,
            module,
            file,
            root,
            token,
        })
    }

    /// Converts a semantic span to a canonical tooling location.
    fn location(&self, span: argui_dsl_syntax::Span) -> Option<Location> {
        Some(Location {
            path: self.file_path(span.file)?.into(),
            start: span.range.start().into(),
            end: span.range.end().into(),
        })
    }
}

/// Chooses a non-trivia token adjacent to the requested byte offset.
fn token_at(root: &SyntaxNode, offset: u32) -> Option<SyntaxToken> {
    let offset = TextSize::from(offset);
    let token = root
        .token_at_offset(offset)
        .right_biased()
        .or_else(|| root.token_at_offset(offset).left_biased())?;
    if !token.kind().is_trivia() {
        return Some(token);
    }
    token
        .prev_token()
        .filter(|candidate| !candidate.kind().is_trivia())
        .or_else(|| {
            token
                .next_token()
                .filter(|candidate| !candidate.kind().is_trivia())
        })
}

/// Returns whether a range contains a UTF-8 byte offset.
fn contains(range: TextRange, offset: u32) -> bool {
    let offset = TextSize::from(offset);
    range.start() <= offset && offset <= range.end()
}

/// Returns the direct first identifier token of one CST node.
fn direct_name(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|item| item.into_token())
        .find(|token| matches!(token.kind(), SyntaxKind::Ident | SyntaxKind::ThemeName))
        .map(|token| token.text().to_owned())
}
