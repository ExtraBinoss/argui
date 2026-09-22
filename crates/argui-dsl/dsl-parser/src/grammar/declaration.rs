use argui_dsl_syntax::SyntaxKind;

use crate::parser::Parser;

use super::{component, expression};

/// Parses one optional-export top-level declaration.
pub(super) fn declaration(parser: &mut Parser<'_>) {
    let exported = parser.at(SyntaxKind::ExportKw);
    let kind = if exported {
        parser.nth_kind(1)
    } else {
        parser.kind()
    };
    match kind {
        SyntaxKind::ImportKw => import(parser),
        SyntaxKind::StructKw => aggregate(parser, SyntaxKind::StructDecl, true),
        SyntaxKind::EnumKw => aggregate(parser, SyntaxKind::EnumDecl, false),
        SyntaxKind::ComponentKw => component::component(parser),
        SyntaxKind::ThemeKw => theme(parser),
        SyntaxKind::StyleKw => style(parser),
        SyntaxKind::EffectKw => effect(parser),
        _ => parser.recover("expected import, struct, enum, component, theme, style, or effect"),
    }
}

/// Parses a named import with optional aliases and a string source.
fn import(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::ImportDecl);
    parser.expect(SyntaxKind::ImportKw, "expected `import`");
    parser.start(SyntaxKind::ImportList);
    parser.expect(SyntaxKind::LBrace, "expected `{` after `import`");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        parser.start(SyntaxKind::ImportItem);
        parser.expect(SyntaxKind::Ident, "expected imported name");
        if parser.at(SyntaxKind::AsKw) {
            parser.bump();
            parser.expect(SyntaxKind::Ident, "expected import alias");
        }
        parser.finish();
        if parser.at(SyntaxKind::Comma) {
            parser.bump();
        } else if !parser.at(SyntaxKind::RBrace) {
            parser.error(
                "expected `,` or `}` in import list",
                [SyntaxKind::Comma, SyntaxKind::RBrace],
            );
            if parser.at(SyntaxKind::Ident) {
                continue;
            }
            parser.recover("invalid import item");
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after imports");
    parser.finish();
    parser.expect(SyntaxKind::FromKw, "expected `from` after import list");
    parser.expect(SyntaxKind::String, "expected import source string");
    if parser.at(SyntaxKind::Semicolon) {
        parser.bump();
    }
    parser.finish();
}

/// Parses a struct field list or enum variant list.
fn aggregate(parser: &mut Parser<'_>, node: SyntaxKind, fields: bool) {
    parser.start(node);
    if parser.at(SyntaxKind::ExportKw) {
        parser.bump();
    }
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected declaration name");
    parser.expect(SyntaxKind::LBrace, "expected `{` after declaration name");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        parser.start(if fields {
            SyntaxKind::FieldDecl
        } else {
            SyntaxKind::VariantDecl
        });
        if !parser.expect(SyntaxKind::Ident, "expected member name") {
            parser.recover("invalid aggregate member");
        }
        if fields {
            parser.expect(SyntaxKind::Colon, "expected `:` after field name");
            type_ref(parser);
        }
        if parser.at(SyntaxKind::Eq) {
            parser.bump();
            expression::expression(parser);
        }
        if parser.at(SyntaxKind::Comma) || parser.at(SyntaxKind::Semicolon) {
            parser.bump();
        }
        parser.finish();
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after declaration members");
    parser.finish();
}

/// Parses a theme declaration with typed tokens and named modes.
fn theme(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::ThemeDecl);
    optional_export_and_keyword(parser, SyntaxKind::ThemeKw);
    parser.expect(SyntaxKind::Ident, "expected theme name");
    parser.expect(SyntaxKind::LBrace, "expected `{` after theme name");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        if parser.at(SyntaxKind::ThemeName) {
            parser.start(SyntaxKind::ThemeTokenDecl);
            parser.bump();
            if parser.at(SyntaxKind::Colon) {
                parser.bump();
                type_ref(parser);
            }
            parser.expect(SyntaxKind::Eq, "expected `=` in theme token");
            expression::expression(parser);
            parser.finish();
        } else if parser.at(SyntaxKind::Ident) && parser.nth_kind(1) == SyntaxKind::LBrace {
            parser.start(SyntaxKind::ThemeModeDecl);
            parser.bump();
            component::property_block(parser);
            parser.finish();
        } else {
            parser.recover("expected a theme token or mode");
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after theme");
    parser.finish();
}

/// Parses a named style targeting a schema/component type.
fn style(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::StyleDecl);
    optional_export_and_keyword(parser, SyntaxKind::StyleKw);
    parser.expect(SyntaxKind::Ident, "expected style name");
    parser.expect(SyntaxKind::ForKw, "expected `for` after style name");
    type_ref(parser);
    component::property_block(parser);
    parser.finish();
}

/// Parses an effect declaration referencing external WGSL and typed parameters.
fn effect(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::EffectDecl);
    optional_export_and_keyword(parser, SyntaxKind::EffectKw);
    parser.expect(SyntaxKind::Ident, "expected effect name");
    parser.expect(SyntaxKind::LBrace, "expected `{` after effect name");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        if parser.at(SyntaxKind::ShaderKw)
            || (parser.at(SyntaxKind::Ident) && parser.text() == "damage")
        {
            parser.start(SyntaxKind::PropertyAssignment);
            parser.bump();
            parser.expect(SyntaxKind::Colon, "expected `:` after effect setting");
            expression::expression(parser);
            parser.finish();
        } else if parser.at(SyntaxKind::ParameterKw) {
            parser.start(SyntaxKind::EffectParameterDecl);
            parser.bump();
            parser.expect(SyntaxKind::Ident, "expected parameter name");
            parser.expect(SyntaxKind::Colon, "expected `:` after parameter name");
            type_ref(parser);
            if parser.at(SyntaxKind::Eq) {
                parser.bump();
                expression::expression(parser);
            }
            parser.finish();
        } else {
            parser.recover("expected `shader`, `damage`, or `parameter` in effect");
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after effect");
    parser.finish();
}

/// Parses optional export syntax followed by a required declaration keyword.
fn optional_export_and_keyword(parser: &mut Parser<'_>, keyword: SyntaxKind) {
    if parser.at(SyntaxKind::ExportKw) {
        parser.bump();
    }
    parser.expect(keyword, "expected declaration keyword");
}

/// Parses a possibly generic and qualified type reference.
pub(super) fn type_ref(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::TypeRef);
    parser.expect(SyntaxKind::Ident, "expected type name");
    if parser.at(SyntaxKind::Lt) {
        parser.bump();
        type_ref(parser);
        while parser.at(SyntaxKind::Comma) {
            parser.bump();
            type_ref(parser);
        }
        parser.expect(SyntaxKind::Gt, "expected `>` after generic type arguments");
    }
    if parser.at(SyntaxKind::Question) {
        parser.bump();
    }
    parser.finish();
}
