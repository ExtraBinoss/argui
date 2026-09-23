use argui_dsl_syntax::SyntaxKind;

use crate::parser::Parser;

/// Parses a complete expression until a natural declarative boundary.
pub(crate) fn expression(parser: &mut Parser<'_>) {
    expression_until(
        parser,
        &[
            SyntaxKind::Comma,
            SyntaxKind::Semicolon,
            SyntaxKind::RBrace,
            SyntaxKind::RParen,
            SyntaxKind::RBracket,
        ],
    );
}

/// Parses an expression while treating `stops` as additional terminators.
///
/// * `parser` — active lossless parser.
/// * `stops` — context-specific tokens that cannot belong to this expression.
pub(crate) fn expression_until(parser: &mut Parser<'_>, stops: &[SyntaxKind]) {
    parser.start(SyntaxKind::Expr);
    if at_stop(parser, stops) {
        parser.error("expected expression", expression_starts());
    } else {
        binary(parser, 0, stops);
    }
    parser.finish();
}

/// Parses a precedence-climbing binary and conditional expression.
fn binary(parser: &mut Parser<'_>, minimum_binding_power: u8, stops: &[SyntaxKind]) {
    let checkpoint = parser.checkpoint();
    prefix(parser, stops);
    loop {
        if at_stop(parser, stops) {
            break;
        }
        if parser.at(SyntaxKind::Question) && minimum_binding_power == 0 {
            parser.start_at(checkpoint, SyntaxKind::ConditionalExpr);
            parser.bump();
            binary(parser, 0, &[SyntaxKind::Colon]);
            parser.expect(SyntaxKind::Colon, "expected `:` in conditional expression");
            binary(parser, 0, stops);
            parser.finish();
            break;
        }
        let Some((left, right)) = binding_power(parser.kind()) else {
            break;
        };
        if left < minimum_binding_power {
            break;
        }
        parser.start_at(checkpoint, SyntaxKind::BinaryExpr);
        parser.bump();
        binary(parser, right, stops);
        parser.finish();
    }
}

/// Parses a prefix atom followed by member access and calls.
fn prefix(parser: &mut Parser<'_>, stops: &[SyntaxKind]) {
    if matches!(
        parser.kind(),
        SyntaxKind::Bang | SyntaxKind::Minus | SyntaxKind::Plus
    ) {
        parser.start(SyntaxKind::UnaryExpr);
        parser.bump();
        binary(parser, 13, stops);
        parser.finish();
        return;
    }
    let checkpoint = parser.checkpoint();
    match parser.kind() {
        SyntaxKind::Number
        | SyntaxKind::String
        | SyntaxKind::TrueKw
        | SyntaxKind::FalseKw
        | SyntaxKind::NullKw => {
            parser.start(SyntaxKind::LiteralExpr);
            parser.bump();
            parser.finish();
        }
        SyntaxKind::Hash => {
            parser.start(SyntaxKind::LiteralExpr);
            parser.bump();
            if matches!(parser.kind(), SyntaxKind::Number | SyntaxKind::Ident) {
                parser.bump();
            } else {
                parser.error("expected hexadecimal color digits", [SyntaxKind::Number]);
            }
            parser.finish();
        }
        SyntaxKind::Ident | SyntaxKind::ThemeName => {
            parser.start(SyntaxKind::PathExpr);
            parser.bump();
            parser.finish();
        }
        SyntaxKind::LParen => {
            parser.bump();
            binary(parser, 0, &[SyntaxKind::RParen]);
            parser.expect(SyntaxKind::RParen, "expected `)` after expression");
        }
        SyntaxKind::LBracket => array(parser),
        _ => {
            parser.error("expected expression", expression_starts());
            if !at_stop(parser, stops) && !parser.at(SyntaxKind::Eof) {
                parser.recover("invalid expression token");
            }
            return;
        }
    }
    loop {
        if parser.at(SyntaxKind::LBrace) && !stops.contains(&SyntaxKind::LBrace) {
            parser.start_at(checkpoint, SyntaxKind::StructExpr);
            parser.bump();
            while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
                parser.start(SyntaxKind::StructFieldExpr);
                parser.expect(SyntaxKind::Ident, "expected struct field name");
                parser.expect(SyntaxKind::Colon, "expected `:` after struct field name");
                expression_until(parser, &[SyntaxKind::Comma, SyntaxKind::RBrace]);
                parser.finish();
                if parser.at(SyntaxKind::Comma) {
                    parser.bump();
                } else {
                    break;
                }
            }
            parser.expect(SyntaxKind::RBrace, "expected `}` after struct literal");
            parser.finish();
        } else if parser.at(SyntaxKind::Dot) {
            parser.start_at(checkpoint, SyntaxKind::MemberExpr);
            parser.bump();
            parser.expect(SyntaxKind::Ident, "expected member name after `.`");
            parser.finish();
        } else if parser.at(SyntaxKind::LBracket) {
            parser.start_at(checkpoint, SyntaxKind::IndexExpr);
            parser.bump();
            expression_until(parser, &[SyntaxKind::RBracket]);
            parser.expect(SyntaxKind::RBracket, "expected `]` after index");
            parser.finish();
        } else if parser.at(SyntaxKind::LParen) {
            parser.start_at(checkpoint, SyntaxKind::CallExpr);
            arguments(parser);
            parser.finish();
        } else {
            break;
        }
    }
}

/// Parses an array literal.
fn array(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::ArrayExpr);
    parser.bump();
    while !parser.at(SyntaxKind::RBracket) && !parser.at(SyntaxKind::Eof) {
        expression_until(parser, &[SyntaxKind::Comma, SyntaxKind::RBracket]);
        if parser.at(SyntaxKind::Comma) {
            parser.bump();
        } else {
            break;
        }
    }
    parser.expect(SyntaxKind::RBracket, "expected `]` after array");
    parser.finish();
}

/// Parses a function-call argument list.
fn arguments(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::ArgumentList);
    parser.bump();
    while !parser.at(SyntaxKind::RParen) && !parser.at(SyntaxKind::Eof) {
        expression_until(parser, &[SyntaxKind::Comma, SyntaxKind::RParen]);
        if parser.at(SyntaxKind::Comma) {
            parser.bump();
        } else {
            break;
        }
    }
    parser.expect(SyntaxKind::RParen, "expected `)` after arguments");
    parser.finish();
}

/// Returns operator binding powers for left-associative expressions.
fn binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    Some(match kind {
        SyntaxKind::OrOr => (1, 2),
        SyntaxKind::AndAnd => (3, 4),
        SyntaxKind::EqEq | SyntaxKind::BangEq => (5, 6),
        SyntaxKind::Lt | SyntaxKind::LtEq | SyntaxKind::Gt | SyntaxKind::GtEq => (7, 8),
        SyntaxKind::Plus | SyntaxKind::Minus => (9, 10),
        SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => (11, 12),
        _ => return None,
    })
}

/// Returns whether the current token terminates this expression context.
fn at_stop(parser: &Parser<'_>, stops: &[SyntaxKind]) -> bool {
    parser.at(SyntaxKind::Eof) || stops.contains(&parser.kind())
}

/// Returns token kinds accepted at the beginning of an expression.
fn expression_starts() -> [SyntaxKind; 11] {
    [
        SyntaxKind::Ident,
        SyntaxKind::ThemeName,
        SyntaxKind::Number,
        SyntaxKind::String,
        SyntaxKind::TrueKw,
        SyntaxKind::FalseKw,
        SyntaxKind::NullKw,
        SyntaxKind::Hash,
        SyntaxKind::LParen,
        SyntaxKind::LBracket,
        SyntaxKind::Bang,
    ]
}
