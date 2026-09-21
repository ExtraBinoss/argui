use argui_dsl_syntax::{SyntaxKind, TextRange, TextSize};
use rowan::{Checkpoint, GreenNodeBuilder};

use crate::{Parse, ParseDiagnostic, grammar, lexer};

pub(crate) struct Parser<'a> {
    source: &'a str,
    tokens: Vec<lexer::Token>,
    cursor: usize,
    builder: GreenNodeBuilder<'static>,
    diagnostics: Vec<ParseDiagnostic>,
}

impl<'a> Parser<'a> {
    /// Creates a parser after lossless tokenization.
    pub(crate) fn new(source: &'a str) -> Self {
        let lexed = lexer::lex(source);
        Self {
            source,
            tokens: lexed.tokens,
            cursor: 0,
            builder: GreenNodeBuilder::new(),
            diagnostics: lexed.diagnostics,
        }
    }

    /// Parses the root and returns the immutable green tree.
    pub(crate) fn parse(mut self) -> Parse {
        // Root must own leading trivia; `start` intentionally emits trivia
        // before opening ordinary grammar nodes.
        self.builder
            .start_node(rowan::SyntaxKind(SyntaxKind::Root as u16));
        grammar::root(&mut self);
        while !self.at(SyntaxKind::Eof) {
            self.bump();
        }
        self.finish();
        Parse {
            green: self.builder.finish(),
            diagnostics: self.diagnostics,
        }
    }

    /// Returns the current raw token kind, including trivia.
    pub(crate) fn raw_kind(&self) -> SyntaxKind {
        self.tokens
            .get(self.cursor)
            .map_or(SyntaxKind::Eof, |token| token.kind)
    }

    /// Returns the next non-trivia token kind.
    pub(crate) fn kind(&self) -> SyntaxKind {
        self.nth_kind(0)
    }

    /// Returns the spelling of the next non-trivia token, or an empty string at EOF.
    #[must_use]
    pub(crate) fn text(&self) -> &str {
        self.tokens
            .iter()
            .skip(self.cursor)
            .find(|token| !token.kind.is_trivia())
            .map_or("", |token| {
                &self.source[std::ops::Range::<usize>::from(token.range)]
            })
    }

    /// Returns the `offset`th non-trivia token kind.
    pub(crate) fn nth_kind(&self, offset: usize) -> SyntaxKind {
        self.tokens
            .iter()
            .skip(self.cursor)
            .filter(|token| !token.kind.is_trivia())
            .nth(offset)
            .map_or(SyntaxKind::Eof, |token| token.kind)
    }

    /// Returns whether the next non-trivia token has `kind`.
    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.kind() == kind
    }

    /// Emits trivia preceding the next grammar token.
    pub(crate) fn trivia(&mut self) {
        while self.raw_kind().is_trivia() {
            self.bump_raw();
        }
    }

    /// Emits the current non-trivia token and its preceding trivia.
    pub(crate) fn bump(&mut self) {
        self.trivia();
        self.bump_raw();
    }

    /// Emits a raw token without skipping trivia.
    fn bump_raw(&mut self) {
        let Some(token) = self.tokens.get(self.cursor) else {
            return;
        };
        let range = std::ops::Range::<usize>::from(token.range);
        self.builder
            .token(rowan::SyntaxKind(token.kind as u16), &self.source[range]);
        self.cursor += 1;
    }

    /// Consumes `kind`, inserting a zero-width missing token and diagnostic otherwise.
    pub(crate) fn expect(&mut self, kind: SyntaxKind, message: &'static str) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            self.error(message, [kind]);
            self.builder
                .token(rowan::SyntaxKind(SyntaxKind::Missing as u16), "");
            false
        }
    }

    /// Starts a new CST node after emitting leading trivia.
    pub(crate) fn start(&mut self, kind: SyntaxKind) {
        self.trivia();
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
    }

    /// Finishes the current CST node.
    pub(crate) fn finish(&mut self) {
        self.builder.finish_node();
    }

    /// Captures a builder position for wrapping an already emitted expression.
    pub(crate) fn checkpoint(&mut self) -> Checkpoint {
        self.trivia();
        self.builder.checkpoint()
    }

    /// Starts a node at an earlier checkpoint.
    pub(crate) fn start_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder
            .start_node_at(checkpoint, rowan::SyntaxKind(kind as u16));
    }

    /// Emits an error node containing one unexpected token so parsing progresses.
    pub(crate) fn recover(&mut self, message: impl Into<String>) {
        self.error(message, []);
        if !self.at(SyntaxKind::Eof) {
            self.start(SyntaxKind::Error);
            self.bump();
            self.finish();
        }
    }

    /// Records a diagnostic at the next non-trivia token or the end of file.
    pub(crate) fn error(
        &mut self,
        message: impl Into<String>,
        expected: impl IntoIterator<Item = SyntaxKind>,
    ) {
        self.diagnostics.push(ParseDiagnostic::new(
            message,
            self.current_range(),
            expected,
        ));
    }

    /// Returns the range of the next non-trivia token.
    fn current_range(&self) -> TextRange {
        self.tokens
            .iter()
            .skip(self.cursor)
            .find(|token| !token.kind.is_trivia())
            .map_or_else(
                || {
                    let end = TextSize::new(self.source.len() as u32);
                    TextRange::empty(end)
                },
                |token| token.range,
            )
    }
}
