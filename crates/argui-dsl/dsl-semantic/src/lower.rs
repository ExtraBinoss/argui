use argui_dsl_parser::Parse;
use argui_dsl_syntax::{
    FileId, Span, SyntaxKind, SyntaxNode, SyntaxToken,
    ast::{AstNode, Declaration},
};

use crate::{Import, ImportItem, SymbolId};

#[derive(Clone)]
pub(crate) struct InputModule {
    pub file: FileId,
    pub path: String,
    pub parse: Parse,
}

#[derive(Clone)]
pub(crate) struct LoweredModule {
    pub file: FileId,
    pub path: String,
    pub imports: Vec<Import>,
    pub definitions: Vec<LoweredDefinition>,
    pub green: argui_dsl_syntax::GreenNode,
}

#[derive(Clone)]
pub(crate) struct LoweredDefinition {
    pub id: SymbolId,
    pub name: String,
    pub exported: bool,
    pub span: Span,
    pub kind: LoweredKind,
    pub syntax: SyntaxNode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoweredKind {
    Struct,
    Enum,
    Component,
    Theme,
    Style,
    Effect,
}

/// Extracts declaration shells while preserving source syntax for later typed passes.
pub(crate) fn module(input: InputModule) -> LoweredModule {
    let tree = input.parse.tree();
    let mut imports = Vec::new();
    let mut definitions = Vec::new();
    for declaration in tree.declarations() {
        match declaration {
            Declaration::Import(import) => imports.push(lower_import(input.file, import.syntax())),
            declaration => {
                let syntax = declaration.syntax().clone();
                let Some((kind, name, discriminator)) = definition_header(&declaration) else {
                    continue;
                };
                let exported =
                    direct_tokens(&syntax).any(|token| token.kind() == SyntaxKind::ExportKw);
                definitions.push(LoweredDefinition {
                    id: SymbolId::derive(&input.path, discriminator, &name),
                    name,
                    exported,
                    span: Span::new(input.file, syntax.text_range()),
                    kind,
                    syntax,
                });
            }
        }
    }
    LoweredModule {
        file: input.file,
        path: input.path,
        imports,
        definitions,
        green: input.parse.green(),
    }
}

/// Returns the semantic kind, name, and ID discriminator for a declaration.
fn definition_header(declaration: &Declaration) -> Option<(LoweredKind, String, &'static str)> {
    let (kind, discriminator) = match declaration {
        Declaration::Struct(_) => (LoweredKind::Struct, "struct"),
        Declaration::Enum(_) => (LoweredKind::Enum, "enum"),
        Declaration::Component(_) => (LoweredKind::Component, "component"),
        Declaration::Theme(_) => (LoweredKind::Theme, "theme"),
        Declaration::Style(_) => (LoweredKind::Style, "style"),
        Declaration::Effect(_) => (LoweredKind::Effect, "effect"),
        Declaration::Import(_) => return None,
    };
    let name = direct_tokens(declaration.syntax())
        .find(|token| token.kind() == SyntaxKind::Ident)?
        .text()
        .to_string();
    Some((kind, name, discriminator))
}

/// Converts one lossless import declaration into semantic import metadata.
fn lower_import(file: FileId, syntax: &SyntaxNode) -> Import {
    let source = syntax
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::String)
        .map(|token| unquote(token.text()))
        .unwrap_or_default();
    let items = syntax
        .children()
        .find(|node| node.kind() == SyntaxKind::ImportList)
        .into_iter()
        .flat_map(|list| list.children())
        .filter(|item| item.kind() == SyntaxKind::ImportItem)
        .filter_map(|item| {
            let names = direct_tokens(&item)
                .filter(|token| token.kind() == SyntaxKind::Ident)
                .collect::<Vec<_>>();
            let name = names.first()?.text().to_string();
            let alias = names
                .get(1)
                .map_or_else(|| name.clone(), |token| token.text().to_string());
            Some(ImportItem {
                name,
                alias,
                span: Span::new(file, item.text_range()),
            })
        })
        .collect();
    Import {
        source,
        items,
        span: Span::new(file, syntax.text_range()),
    }
}

/// Iterates over non-recursive tokens directly owned by a syntax node.
pub(crate) fn direct_tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> + '_ {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
}

/// Returns all non-trivia text belonging to a syntax node.
pub(crate) fn compact_text(node: &SyntaxNode) -> String {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia() && token.kind() != SyntaxKind::Missing)
        .map(|token| token.text().to_string())
        .collect()
}

/// Removes quotes from a parser-retained string token.
pub(crate) fn unquote(text: &str) -> String {
    text.strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(text)
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}
