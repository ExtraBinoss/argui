mod component;
mod declaration;
pub(crate) mod expression;

use argui_dsl_syntax::SyntaxKind;

use crate::parser::Parser;

/// Parses all top-level declarations with token-by-token recovery.
pub(crate) fn root(parser: &mut Parser<'_>) {
    while !parser.at(SyntaxKind::Eof) {
        if parser.kind().is_declaration_start() {
            declaration::declaration(parser);
        } else {
            parser.recover("expected a top-level declaration");
        }
    }
    parser.trivia();
}
