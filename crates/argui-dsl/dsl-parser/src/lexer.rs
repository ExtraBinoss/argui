use argui_dsl_syntax::{SyntaxKind, TextRange, TextSize};

use crate::ParseDiagnostic;

/// One lossless lexical token referencing its original byte range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub range: TextRange,
}

/// Complete lossless lexical output and invalid-token diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Tokenizes source without dropping whitespace, comments, or invalid bytes.
///
/// * `source` — UTF-8 Argui source text.
#[must_use]
pub fn lex(source: &str) -> Lexed {
    let mut lexer = Lexer {
        source,
        offset: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
    };
    while lexer.offset < source.len() {
        lexer.next_token();
    }
    Lexed {
        tokens: lexer.tokens,
        diagnostics: lexer.diagnostics,
    }
}

struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<ParseDiagnostic>,
}

impl Lexer<'_> {
    /// Scans and emits one token beginning at the current byte offset.
    fn next_token(&mut self) {
        let start = self.offset;
        let rest = &self.source[start..];
        let first = rest.chars().next().expect("offset is inside source");
        let kind = if first.is_whitespace() {
            self.take_while(char::is_whitespace);
            SyntaxKind::Whitespace
        } else if rest.starts_with("//") {
            self.offset += 2;
            self.take_while(|character| character != '\n');
            SyntaxKind::LineComment
        } else if rest.starts_with("/*") {
            self.block_comment(start)
        } else if first == '"' {
            self.string(start)
        } else if rest.starts_with("--") && rest[2..].chars().next().is_some_and(is_ident_start) {
            self.offset += 2;
            self.take_while(is_ident_continue);
            SyntaxKind::ThemeName
        } else if is_ident_start(first) {
            self.offset += first.len_utf8();
            self.take_while(is_ident_continue);
            keyword(&self.source[start..self.offset]).unwrap_or(SyntaxKind::Ident)
        } else if first.is_ascii_digit()
            || (first == '.'
                && rest[1..]
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit()))
        {
            self.number();
            SyntaxKind::Number
        } else if let Some((kind, length)) = punctuation(rest) {
            self.offset += length;
            kind
        } else {
            self.offset += first.len_utf8();
            let range = range(start, self.offset);
            self.diagnostics.push(ParseDiagnostic::new(
                format!("unexpected character `{first}`"),
                range,
                [],
            ));
            SyntaxKind::ErrorToken
        };
        self.tokens.push(Token {
            kind,
            range: range(start, self.offset),
        });
    }

    /// Consumes a possibly nested block comment and diagnoses a missing terminator.
    fn block_comment(&mut self, start: usize) -> SyntaxKind {
        self.offset += 2;
        let mut depth = 1_u32;
        while self.offset < self.source.len() {
            let rest = &self.source[self.offset..];
            if rest.starts_with("/*") {
                depth += 1;
                self.offset += 2;
            } else if rest.starts_with("*/") {
                depth -= 1;
                self.offset += 2;
                if depth == 0 {
                    return SyntaxKind::BlockComment;
                }
            } else {
                self.offset += rest.chars().next().map_or(1, char::len_utf8);
            }
        }
        self.diagnostics.push(ParseDiagnostic::new(
            "unterminated block comment",
            range(start, self.offset),
            [],
        ));
        SyntaxKind::BlockComment
    }

    /// Consumes an escaped string while retaining unterminated input as one token.
    fn string(&mut self, start: usize) -> SyntaxKind {
        self.offset += 1;
        let mut escaped = false;
        while self.offset < self.source.len() {
            let character = self.source[self.offset..]
                .chars()
                .next()
                .expect("offset is inside string source");
            self.offset += character.len_utf8();
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                return SyntaxKind::String;
            }
        }
        self.diagnostics.push(ParseDiagnostic::new(
            "unterminated string literal",
            range(start, self.offset),
            [SyntaxKind::String],
        ));
        SyntaxKind::String
    }

    /// Consumes a decimal/scientific number and an optional unit suffix.
    fn number(&mut self) {
        let mut seen_exponent = false;
        let mut allow_exponent_sign = false;
        while let Some(character) = self.source[self.offset..].chars().next() {
            let accepted = if character.is_ascii_digit() || character == '_' || character == '.' {
                allow_exponent_sign = false;
                true
            } else if matches!(character, 'e' | 'E') && !seen_exponent {
                seen_exponent = true;
                allow_exponent_sign = true;
                true
            } else if matches!(character, '+' | '-') && allow_exponent_sign {
                allow_exponent_sign = false;
                true
            } else {
                character.is_alphabetic() || character == '%'
            };
            if !accepted {
                break;
            }
            self.offset += character.len_utf8();
        }
    }

    /// Consumes characters while `predicate` returns true.
    fn take_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(character) = self.source[self.offset..].chars().next() {
            if !predicate(character) {
                break;
            }
            self.offset += character.len_utf8();
        }
    }
}

/// Returns whether a character may begin an identifier.
fn is_ident_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

/// Returns whether a character may continue an identifier or kebab-case property name.
fn is_ident_continue(character: char) -> bool {
    character == '_' || character == '-' || character.is_alphanumeric()
}

/// Maps reserved words while leaving contextual names as identifiers.
fn keyword(text: &str) -> Option<SyntaxKind> {
    Some(match text {
        "import" => SyntaxKind::ImportKw,
        "from" => SyntaxKind::FromKw,
        "as" => SyntaxKind::AsKw,
        "export" => SyntaxKind::ExportKw,
        "struct" => SyntaxKind::StructKw,
        "enum" => SyntaxKind::EnumKw,
        "component" => SyntaxKind::ComponentKw,
        "theme" => SyntaxKind::ThemeKw,
        "style" => SyntaxKind::StyleKw,
        "effect" => SyntaxKind::EffectKw,
        "for" => SyntaxKind::ForKw,
        "in" => SyntaxKind::InKw,
        "key" => SyntaxKind::KeyKw,
        "if" => SyntaxKind::IfKw,
        "else" => SyntaxKind::ElseKw,
        "property" => SyntaxKind::PropertyKw,
        "out" => SyntaxKind::OutKw,
        "in-out" => SyntaxKind::InOutKw,
        "private" => SyntaxKind::PrivateKw,
        "callback" => SyntaxKind::CallbackKw,
        "slot" => SyntaxKind::SlotKw,
        "on" => SyntaxKind::OnKw,
        "states" => SyntaxKind::StatesKw,
        "when" => SyntaxKind::WhenKw,
        "animate" => SyntaxKind::AnimateKw,
        "parameter" => SyntaxKind::ParameterKw,
        "shader" => SyntaxKind::ShaderKw,
        "true" => SyntaxKind::TrueKw,
        "false" => SyntaxKind::FalseKw,
        "null" => SyntaxKind::NullKw,
        "let" => SyntaxKind::LetKw,
        "return" => SyntaxKind::ReturnKw,
        _ => return None,
    })
}

/// Recognizes longest-match punctuation and operators.
fn punctuation(source: &str) -> Option<(SyntaxKind, usize)> {
    let pair = [
        ("<=>", SyntaxKind::TwoWay),
        ("->", SyntaxKind::Arrow),
        ("=>", SyntaxKind::FatArrow),
        ("==", SyntaxKind::EqEq),
        ("!=", SyntaxKind::BangEq),
        ("<=", SyntaxKind::LtEq),
        (">=", SyntaxKind::GtEq),
        ("&&", SyntaxKind::AndAnd),
        ("||", SyntaxKind::OrOr),
        ("+=", SyntaxKind::PlusEq),
        ("-=", SyntaxKind::MinusEq),
        ("*=", SyntaxKind::StarEq),
        ("/=", SyntaxKind::SlashEq),
    ];
    for (text, kind) in pair {
        if source.starts_with(text) {
            return Some((kind, text.len()));
        }
    }
    let character = source.chars().next()?;
    let kind = match character {
        '{' => SyntaxKind::LBrace,
        '}' => SyntaxKind::RBrace,
        '(' => SyntaxKind::LParen,
        ')' => SyntaxKind::RParen,
        '[' => SyntaxKind::LBracket,
        ']' => SyntaxKind::RBracket,
        ',' => SyntaxKind::Comma,
        ':' => SyntaxKind::Colon,
        ';' => SyntaxKind::Semicolon,
        '.' => SyntaxKind::Dot,
        '#' => SyntaxKind::Hash,
        '@' => SyntaxKind::At,
        '+' => SyntaxKind::Plus,
        '-' => SyntaxKind::Minus,
        '*' => SyntaxKind::Star,
        '/' => SyntaxKind::Slash,
        '%' => SyntaxKind::Percent,
        '!' => SyntaxKind::Bang,
        '=' => SyntaxKind::Eq,
        '<' => SyntaxKind::Lt,
        '>' => SyntaxKind::Gt,
        '?' => SyntaxKind::Question,
        _ => return None,
    };
    Some((kind, character.len_utf8()))
}

/// Builds a Rowan byte range from native offsets.
fn range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::new(start as u32), TextSize::new(end as u32))
}
