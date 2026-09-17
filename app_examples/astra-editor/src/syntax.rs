//! Cached Rust syntax highlighting used by the editable text surface.

#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;

use argui::{
    core::Color,
    text::{TextContent, TextSpan, TextSpanStyle},
};
#[cfg(not(target_arch = "wasm32"))]
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

const MAX_HIGHLIGHT_BYTES: usize = 512 * 1024;
const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "boolean",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "escape",
    "function",
    "function.builtin",
    "function.macro",
    "function.method",
    "keyword",
    "keyword.control",
    "keyword.function",
    "keyword.operator",
    "label",
    "number",
    "operator",
    "property",
    "punctuation.bracket",
    "punctuation.delimiter",
    "string",
    "string.special",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static RUST_HIGHLIGHTER: RefCell<Option<RustHighlighter>> =
        RefCell::new(RustHighlighter::new());
}

/// Light and dark rich-text variants for one Rust source snapshot.
#[derive(Clone, Debug)]
pub(crate) struct HighlightedCode {
    light: TextContent,
    dark: TextContent,
}

impl HighlightedCode {
    /// Returns the rich content matching `dark` mode.
    #[must_use]
    pub(crate) fn content(&self, dark: bool) -> TextContent {
        if dark {
            self.dark.clone()
        } else {
            self.light.clone()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct RustHighlighter {
    engine: Highlighter,
    config: HighlightConfiguration,
}

#[cfg(not(target_arch = "wasm32"))]
impl RustHighlighter {
    /// Creates the reusable Rust parser and configured highlight query.
    fn new() -> Option<Self> {
        let mut config = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .ok()?;
        config.configure(HIGHLIGHT_NAMES);
        Some(Self {
            engine: Highlighter::new(),
            config,
        })
    }

    /// Parses `source` once and returns styled source fragments.
    fn highlight(&mut self, source: &str) -> Option<Vec<(usize, usize, Option<usize>)>> {
        let events = self
            .engine
            .highlight(&self.config, source.as_bytes(), None, |_| None)
            .ok()?;
        let mut stack = Vec::new();
        let mut fragments = Vec::new();
        for event in events {
            match event.ok()? {
                HighlightEvent::HighlightStart(highlight) => stack.push(highlight.0),
                HighlightEvent::HighlightEnd => {
                    stack.pop();
                }
                HighlightEvent::Source { start, end } => {
                    fragments.push((start, end, stack.last().copied()));
                }
            }
        }
        Some(fragments)
    }
}

/// Highlights Rust `source` for both supported themes, or returns a plain-text fallback.
#[must_use]
pub(crate) fn highlight(path: &str, source: &str) -> Option<HighlightedCode> {
    if !path.ends_with(".rs") || source.len() > MAX_HIGHLIGHT_BYTES {
        return None;
    }
    fragments(source).map(|fragments| HighlightedCode {
        light: content(source, &fragments, false),
        dark: content(source, &fragments, true),
    })
}

/// Returns parsed source fragments using Tree-sitter on native platforms.
#[cfg(not(target_arch = "wasm32"))]
fn fragments(source: &str) -> Option<Vec<(usize, usize, Option<usize>)>> {
    RUST_HIGHLIGHTER.with(|highlighter| highlighter.borrow_mut().as_mut()?.highlight(source))
}

/// Returns dependency-free Rust lexical fragments for the WebAssembly build.
#[cfg(target_arch = "wasm32")]
fn fragments(source: &str) -> Option<Vec<(usize, usize, Option<usize>)>> {
    Some(lex_rust(source))
}

/// Tokenizes enough Rust syntax to preserve highlighting in dependency-free Wasm builds.
#[cfg(target_arch = "wasm32")]
fn lex_rust(source: &str) -> Vec<(usize, usize, Option<usize>)> {
    let bytes = source.as_bytes();
    let mut output = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        let capture = if source[start..].starts_with("//") {
            index = source[start..]
                .find('\n')
                .map_or(bytes.len(), |offset| start + offset);
            capture_index("comment")
        } else if source[start..].starts_with("/*") {
            index = source[start + 2..]
                .find("*/")
                .map_or(bytes.len(), |offset| start + 2 + offset + 2);
            capture_index("comment")
        } else if bytes[index] == b'"' {
            index = quoted_end(bytes, index, b'"');
            capture_index("string")
        } else if bytes[index] == b'\'' {
            index = quoted_end(bytes, index, b'\'');
            capture_index(if index.saturating_sub(start) > 1 {
                "string"
            } else {
                "label"
            })
        } else if bytes[index].is_ascii_digit() {
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'_' | b'.'))
            {
                index += 1;
            }
            capture_index("number")
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            identifier_capture(&source[start..index], &source[index..])
        } else if bytes[index].is_ascii_whitespace() {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            None
        } else {
            let character = source[index..]
                .chars()
                .next()
                .expect("lexer index must stay on a UTF-8 boundary");
            index += character.len_utf8();
            match character {
                '(' | ')' | '[' | ']' | '{' | '}' => capture_index("punctuation.bracket"),
                ',' | ';' | ':' | '.' => capture_index("punctuation.delimiter"),
                '+' | '-' | '*' | '/' | '%' | '=' | '!' | '&' | '|' | '^' | '<' | '>' => {
                    capture_index("operator")
                }
                _ => None,
            }
        };
        push_fragment(&mut output, start, index, capture);
    }
    output
}

/// Finds the end of a simple escaped Rust string or character literal.
#[cfg(target_arch = "wasm32")]
fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == quote {
            return index + 1;
        } else {
            index += 1;
        }
    }
    bytes.len()
}

/// Maps an identifier and following source to a syntax capture.
#[cfg(target_arch = "wasm32")]
fn identifier_capture(identifier: &str, rest: &str) -> Option<usize> {
    if matches!(identifier, "true" | "false") {
        return capture_index("boolean");
    }
    if matches!(
        identifier,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    ) {
        return capture_index("keyword");
    }
    if matches!(
        identifier,
        "bool"
            | "char"
            | "f32"
            | "f64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
    ) {
        return capture_index("type.builtin");
    }
    let next = rest.trim_start().as_bytes().first().copied();
    if next == Some(b'!') {
        capture_index("function.macro")
    } else if next == Some(b'(') {
        capture_index("function")
    } else if identifier.chars().next().is_some_and(char::is_uppercase) {
        capture_index("type")
    } else {
        capture_index("variable")
    }
}

/// Coalesces adjacent fragments with the same capture to keep rich text compact.
#[cfg(target_arch = "wasm32")]
fn push_fragment(
    output: &mut Vec<(usize, usize, Option<usize>)>,
    start: usize,
    end: usize,
    capture: Option<usize>,
) {
    if let Some((_, previous_end, previous_capture)) = output.last_mut()
        && *previous_end == start
        && *previous_capture == capture
    {
        *previous_end = end;
    } else {
        output.push((start, end, capture));
    }
}

/// Resolves one known capture name to its stable style index.
#[cfg(target_arch = "wasm32")]
fn capture_index(name: &str) -> Option<usize> {
    HIGHLIGHT_NAMES
        .iter()
        .position(|candidate| *candidate == name)
}

/// Converts parsed source fragments into Argui rich-text spans.
fn content(source: &str, fragments: &[(usize, usize, Option<usize>)], dark: bool) -> TextContent {
    TextContent::rich(fragments.iter().map(|&(start, end, highlight)| {
        let span = TextSpan::new(&source[start..end]);
        highlight.map_or(span.clone(), |highlight| {
            span.style(TextSpanStyle::default().color(color(HIGHLIGHT_NAMES[highlight], dark)))
        })
    }))
}

/// Returns a VS Code-inspired color for one Tree-sitter capture name.
fn color(name: &str, dark: bool) -> Color {
    let rgb = if dark {
        match name {
            "comment" | "comment.documentation" => (106, 153, 85),
            "string" | "string.special" | "escape" => (206, 145, 120),
            "number" | "boolean" => (181, 206, 168),
            "type" | "type.builtin" | "constructor" => (78, 201, 176),
            "function" | "function.builtin" | "function.method" | "function.macro" => {
                (220, 220, 170)
            }
            "variable" | "variable.builtin" | "variable.parameter" | "property" => (156, 220, 254),
            "constant" | "constant.builtin" => (79, 193, 255),
            "attribute" => (197, 134, 192),
            "keyword" | "keyword.control" | "keyword.function" | "keyword.operator" => {
                (197, 134, 192)
            }
            _ => (212, 212, 212),
        }
    } else {
        match name {
            "comment" | "comment.documentation" => (0, 128, 0),
            "string" | "string.special" | "escape" => (163, 21, 21),
            "number" | "boolean" => (9, 134, 88),
            "type" | "type.builtin" | "constructor" => (38, 127, 153),
            "function" | "function.builtin" | "function.method" | "function.macro" => (121, 94, 38),
            "variable" | "variable.builtin" | "variable.parameter" | "property" => (0, 16, 128),
            "constant" | "constant.builtin" => (0, 112, 193),
            "attribute" => (128, 64, 0),
            "keyword" | "keyword.control" | "keyword.function" | "keyword.operator" => {
                (175, 0, 219)
            }
            _ => (31, 31, 31),
        }
    };
    Color::from_srgb8(rgb.0, rgb.1, rgb.2)
}
