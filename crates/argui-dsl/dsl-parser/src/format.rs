use argui_dsl_syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

use crate::parse;

/// Formats source from its lossless CST with deterministic whitespace and import order.
///
/// Invalid syntax is returned unchanged so formatting never destroys recoverable input.
#[must_use]
pub fn format_source(source: &str) -> String {
    let parsed = parse(source);
    if !parsed.diagnostics().is_empty() {
        return source.to_owned();
    }
    let formatted = Formatter::new().format(&parsed.syntax());
    sort_import_blocks(&formatted)
}

/// Stateful token writer which derives line boundaries from CST ownership.
struct Formatter {
    output: String,
    indent: usize,
    line_start: bool,
    pending_newlines: usize,
    previous: Option<SyntaxKind>,
}

impl Formatter {
    /// Creates an empty formatter state.
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            line_start: true,
            pending_newlines: 0,
            previous: None,
        }
    }

    /// Formats every non-whitespace token exactly once.
    fn format(mut self, root: &SyntaxNode) -> String {
        for token in root
            .descendants_with_tokens()
            .filter_map(|item| item.into_token())
        {
            if token.kind() != SyntaxKind::Whitespace {
                self.token(&token);
            }
        }
        self.newlines(1);
        self.flush_newlines();
        self.output
    }

    /// Writes one syntax token and schedules structural line breaks.
    fn token(&mut self, token: &SyntaxToken) {
        let kind = token.kind();
        if kind == SyntaxKind::Eof {
            return;
        }
        if starts_top_level(token) && !self.output.is_empty() {
            self.newlines(2);
        }
        if kind == SyntaxKind::RBrace && multiline_brace(token) {
            self.indent = self.indent.saturating_sub(1);
            self.newlines(1);
        }
        if kind == SyntaxKind::ElseKw && self.previous == Some(SyntaxKind::RBrace) {
            self.pending_newlines = 0;
            self.space();
        }
        if kind == SyntaxKind::LineComment {
            self.comment(token.text(), true);
        } else if kind == SyntaxKind::BlockComment {
            self.comment(token.text(), false);
        } else {
            self.flush_newlines();
            if needs_space(self.previous, kind, self.line_start) {
                self.space();
            }
            self.write(token.text());
        }
        match kind {
            SyntaxKind::LBrace if multiline_brace(token) => {
                self.indent += 1;
                self.newlines(1);
            }
            SyntaxKind::RBrace if !is_import_list(token) => self.newlines(1),
            SyntaxKind::Semicolon => self.newlines(1),
            SyntaxKind::Comma if comma_owns_line(token) => self.newlines(1),
            _ if ends_line_node(token) => self.newlines(1),
            _ => {}
        }
        self.previous = Some(kind);
    }

    /// Writes a comment without altering its text.
    fn comment(&mut self, text: &str, line: bool) {
        self.flush_newlines();
        if !self.line_start {
            self.space();
        }
        self.write(text);
        if line {
            self.newlines(1);
        }
    }

    /// Requests at least `count` line breaks before the next token.
    fn newlines(&mut self, count: usize) {
        self.pending_newlines = self.pending_newlines.max(count);
    }

    /// Materializes requested line breaks and indentation lazily.
    fn flush_newlines(&mut self) {
        if self.pending_newlines == 0 {
            if self.line_start {
                self.write_indent();
            }
            return;
        }
        while self.output.ends_with(' ') {
            self.output.pop();
        }
        while self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output.push('\n');
        if self.pending_newlines > 1 && !self.output.trim().is_empty() {
            self.output.push('\n');
        }
        self.pending_newlines = 0;
        self.line_start = true;
        self.write_indent();
    }

    /// Writes current indentation if the cursor is at a fresh line.
    fn write_indent(&mut self) {
        if self.line_start {
            self.output.push_str(&"    ".repeat(self.indent));
            self.line_start = false;
        }
    }

    /// Writes one literal token fragment.
    fn write(&mut self, text: &str) {
        self.write_indent();
        self.output.push_str(text);
    }

    /// Writes one separating space unless whitespace already exists.
    fn space(&mut self) {
        if !self.line_start && !self.output.ends_with([' ', '\n']) {
            self.output.push(' ');
        }
    }
}

/// Returns whether this token begins a direct root declaration.
fn starts_top_level(token: &SyntaxToken) -> bool {
    token.parent_ancestors().any(|node| {
        is_top_level(node.kind())
            && node.text_range().start() == token.text_range().start()
            && node
                .parent()
                .is_some_and(|parent| parent.kind() == SyntaxKind::Root)
    })
}

/// Returns whether braces owned by this token expand over multiple lines.
fn multiline_brace(token: &SyntaxToken) -> bool {
    matches!(token.kind(), SyntaxKind::LBrace | SyntaxKind::RBrace)
}

/// Returns whether a closing brace belongs to an import list.
fn is_import_list(token: &SyntaxToken) -> bool {
    token
        .parent_ancestors()
        .next()
        .is_some_and(|node| node.kind() == SyntaxKind::ImportList)
}

/// Returns whether a comma separates multiline declaration members.
fn comma_owns_line(token: &SyntaxToken) -> bool {
    token.parent_ancestors().any(|node| {
        matches!(
            node.kind(),
            SyntaxKind::ImportList | SyntaxKind::StructDecl | SyntaxKind::EnumDecl
        )
    })
}

/// Returns whether a completed CST member requires a following line break.
fn ends_line_node(token: &SyntaxToken) -> bool {
    token.parent_ancestors().any(|node| {
        node.text_range().end() == token.text_range().end()
            && matches!(
                node.kind(),
                SyntaxKind::PropertyDecl
                    | SyntaxKind::CallbackDecl
                    | SyntaxKind::SlotDecl
                    | SyntaxKind::EffectParameterDecl
                    | SyntaxKind::PropertyAssignment
                    | SyntaxKind::TwoWayBinding
                    | SyntaxKind::ThemeTokenDecl
            )
    })
}

/// Returns whether a syntax kind represents a root declaration.
fn is_top_level(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ImportDecl
            | SyntaxKind::StructDecl
            | SyntaxKind::EnumDecl
            | SyntaxKind::ComponentDecl
            | SyntaxKind::ThemeDecl
            | SyntaxKind::StyleDecl
            | SyntaxKind::EffectDecl
            | SyntaxKind::FunctionDecl
    )
}

/// Computes deterministic token separation for the grammar's punctuation.
fn needs_space(previous: Option<SyntaxKind>, current: SyntaxKind, line_start: bool) -> bool {
    if line_start || previous.is_none() {
        return false;
    }
    let previous = previous.expect("checked above");
    if matches!(
        current,
        SyntaxKind::Comma
            | SyntaxKind::Semicolon
            | SyntaxKind::Colon
            | SyntaxKind::Dot
            | SyntaxKind::RParen
            | SyntaxKind::RBracket
            | SyntaxKind::RBrace
    ) || matches!(
        previous,
        SyntaxKind::LParen
            | SyntaxKind::LBracket
            | SyntaxKind::Dot
            | SyntaxKind::At
            | SyntaxKind::Hash
    ) {
        return false;
    }
    !matches!(current, SyntaxKind::LParen)
}

/// Sorts comment-free import blocks while leaving all other blocks stable.
fn sort_import_blocks(formatted: &str) -> String {
    let mut blocks = formatted
        .trim()
        .split("\n\n")
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut imports = blocks
        .iter()
        .filter(|block| block.trim_start().starts_with("import "))
        .cloned()
        .collect::<Vec<_>>();
    imports.sort();
    let mut index = 0;
    for block in &mut blocks {
        if block.trim_start().starts_with("import ") {
            *block = imports[index].clone();
            index += 1;
        }
    }
    format!("{}\n", blocks.join("\n\n"))
}
