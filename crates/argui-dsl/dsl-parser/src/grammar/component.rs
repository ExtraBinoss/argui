use argui_dsl_syntax::SyntaxKind;

use crate::parser::Parser;

use super::{declaration::type_ref, expression};

/// Parses a component declaration and its declarative body.
pub(super) fn component(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::ComponentDecl);
    if parser.at(SyntaxKind::ExportKw) {
        parser.bump();
    }
    parser.expect(SyntaxKind::ComponentKw, "expected `component`");
    parser.expect(SyntaxKind::Ident, "expected component name");
    parser.expect(SyntaxKind::LBrace, "expected `{` after component name");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        component_item(parser);
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after component");
    parser.finish();
}

/// Parses one declaration or visual item inside a component.
fn component_item(parser: &mut Parser<'_>) {
    match parser.kind() {
        SyntaxKind::InKw
        | SyntaxKind::OutKw
        | SyntaxKind::InOutKw
        | SyntaxKind::PrivateKw
        | SyntaxKind::PropertyKw => property_decl(parser),
        SyntaxKind::CallbackKw => callback_decl(parser),
        SyntaxKind::SlotKw => slot_decl(parser),
        SyntaxKind::ForKw => repeater(parser),
        SyntaxKind::IfKw => condition(parser),
        SyntaxKind::StatesKw => states(parser),
        SyntaxKind::AnimateKw => animate(parser),
        SyntaxKind::Ident if is_element(parser) => element(parser),
        _ => parser.recover("expected a component declaration, element, repeater, or condition"),
    }
}

/// Parses a directional or private property declaration.
fn property_decl(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::PropertyDecl);
    if matches!(
        parser.kind(),
        SyntaxKind::InKw | SyntaxKind::OutKw | SyntaxKind::InOutKw | SyntaxKind::PrivateKw
    ) {
        parser.bump();
    }
    parser.expect(SyntaxKind::PropertyKw, "expected `property`");
    parser.expect(SyntaxKind::Ident, "expected property name");
    parser.expect(SyntaxKind::Colon, "expected `:` after property name");
    type_ref(parser);
    if parser.at(SyntaxKind::Eq) {
        parser.bump();
        expression::expression(parser);
    }
    parser.finish();
}

/// Parses a typed callback declaration.
fn callback_decl(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::CallbackDecl);
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected callback name");
    parser.expect(SyntaxKind::LParen, "expected `(` after callback name");
    while !parser.at(SyntaxKind::RParen) && !parser.at(SyntaxKind::Eof) {
        parser.start(SyntaxKind::CallbackParameter);
        parser.expect(SyntaxKind::Ident, "expected callback parameter name");
        parser.expect(SyntaxKind::Colon, "expected `:` after callback parameter");
        type_ref(parser);
        parser.finish();
        if parser.at(SyntaxKind::Comma) {
            parser.bump();
        } else {
            break;
        }
    }
    parser.expect(SyntaxKind::RParen, "expected `)` after callback parameters");
    if parser.at(SyntaxKind::Arrow) {
        parser.bump();
        type_ref(parser);
    }
    parser.finish();
}

/// Parses a named component slot declaration.
fn slot_decl(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::SlotDecl);
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected slot name");
    if parser.at(SyntaxKind::LParen) {
        parser.bump();
        while !parser.at(SyntaxKind::RParen) && !parser.at(SyntaxKind::Eof) {
            parser.start(SyntaxKind::SlotParameter);
            parser.expect(SyntaxKind::Ident, "expected slot parameter name");
            parser.expect(SyntaxKind::Colon, "expected `:` after slot parameter");
            type_ref(parser);
            parser.finish();
            if parser.at(SyntaxKind::Comma) {
                parser.bump();
            } else {
                break;
            }
        }
        parser.expect(SyntaxKind::RParen, "expected `)` after slot parameters");
    }
    if parser.at(SyntaxKind::Colon) {
        parser.bump();
        parser.expect(SyntaxKind::Ident, "expected slot kind after `:`");
        if parser.at(SyntaxKind::Ident) && parser.text() == "single" {
            parser.bump();
        }
    }
    if parser.at(SyntaxKind::LBrace) {
        visual_block(parser);
    }
    parser.finish();
}

/// Parses caller-authored content for one named component slot.
fn slot_content(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::SlotContent);
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected supplied slot name");
    visual_block(parser);
    parser.finish();
}

/// Parses one visual element and its properties, events, and children.
pub(super) fn element(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::Element);
    parser.expect(SyntaxKind::Ident, "expected element type");
    if parser.at(SyntaxKind::Hash) {
        parser.start(SyntaxKind::ElementId);
        parser.bump();
        parser.expect(SyntaxKind::Ident, "expected source ID after `#`");
        parser.finish();
    }
    parser.expect(SyntaxKind::LBrace, "expected `{` after element type");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        match parser.kind() {
            SyntaxKind::OnKw => event(parser),
            SyntaxKind::SlotKw => slot_content(parser),
            SyntaxKind::ForKw => repeater(parser),
            SyntaxKind::IfKw => condition(parser),
            SyntaxKind::StatesKw => states(parser),
            SyntaxKind::AnimateKw => animate(parser),
            SyntaxKind::EffectKw => effect_application(parser),
            SyntaxKind::StyleKw => style_application(parser),
            SyntaxKind::Ident if is_element(parser) => element(parser),
            SyntaxKind::Ident
                if matches!(parser.nth_kind(1), SyntaxKind::Colon | SyntaxKind::TwoWay) =>
            {
                assignment(parser);
            }
            SyntaxKind::Ident => slot_reference(parser),
            SyntaxKind::ThemeName => assignment(parser),
            _ => parser.recover("expected an element property, event, or child"),
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after element");
    parser.finish();
}

/// Parses an explicit named style application on an element.
fn style_application(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::StyleApplication);
    parser.bump();
    parser.expect(SyntaxKind::Colon, "expected `:` after `style`");
    parser.expect(SyntaxKind::Ident, "expected style name");
    if parser.at(SyntaxKind::Semicolon) {
        parser.bump();
    }
    parser.finish();
}

/// Parses one named effect and its parameter expressions on a visual element.
fn effect_application(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::EffectApplication);
    parser.bump();
    parser.expect(SyntaxKind::Colon, "expected `:` after `effect`");
    parser.expect(SyntaxKind::Ident, "expected effect name");
    parser.expect(SyntaxKind::LBrace, "expected `{` after effect name");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        if parser.at(SyntaxKind::Ident) && parser.nth_kind(1) == SyntaxKind::Colon {
            assignment(parser);
        } else {
            parser.recover("expected an effect parameter assignment");
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after effect parameters");
    if parser.at(SyntaxKind::Semicolon) {
        parser.bump();
    }
    parser.finish();
}

/// Parses a bare named slot reference nested inside an element body.
fn slot_reference(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::PathExpr);
    parser.bump();
    parser.finish();
}

/// Parses a property assignment or explicit two-way binding.
fn assignment(parser: &mut Parser<'_>) {
    let node = if parser.nth_kind(1) == SyntaxKind::TwoWay {
        SyntaxKind::TwoWayBinding
    } else {
        SyntaxKind::PropertyAssignment
    };
    parser.start(node);
    parser.bump();
    if node == SyntaxKind::TwoWayBinding {
        parser.bump();
    } else {
        parser.expect(
            SyntaxKind::Colon,
            "expected `:` or `<=>` after property name",
        );
    }
    expression::expression(parser);
    if parser.at(SyntaxKind::Semicolon) {
        parser.bump();
    }
    parser.finish();
}

/// Parses an event handler, optional payload names, and its restricted statements.
fn event(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::EventBlock);
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected event name");
    if parser.at(SyntaxKind::LParen) {
        parser.bump();
        while !parser.at(SyntaxKind::RParen) && !parser.at(SyntaxKind::Eof) {
            parser.expect(SyntaxKind::Ident, "expected event parameter name");
            if parser.at(SyntaxKind::Comma) {
                parser.bump();
            } else {
                break;
            }
        }
        parser.expect(SyntaxKind::RParen, "expected `)` after event parameters");
    }
    parser.expect(SyntaxKind::LBrace, "expected `{` after event name");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        handler_statement(parser);
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after event handler");
    parser.finish();
}

/// Parses one handler statement, including a lexical conditional block.
fn handler_statement(parser: &mut Parser<'_>) {
    if parser.at(SyntaxKind::IfKw) {
        parser.start(SyntaxKind::IfStatement);
        parser.bump();
        expression::expression_until(parser, &[SyntaxKind::LBrace]);
        handler_block(parser);
        if parser.at(SyntaxKind::ElseKw) {
            parser.bump();
            handler_block(parser);
        }
        parser.finish();
        return;
    }
    parser.start(SyntaxKind::Statement);
    if parser.at(SyntaxKind::LetKw) || parser.at(SyntaxKind::ReturnKw) {
        let returning = parser.at(SyntaxKind::ReturnKw);
        parser.bump();
        if returning && matches!(parser.kind(), SyntaxKind::Semicolon | SyntaxKind::RBrace) {
            if parser.at(SyntaxKind::Semicolon) {
                parser.bump();
            }
            parser.finish();
            return;
        }
    }
    expression::expression(parser);
    if matches!(
        parser.kind(),
        SyntaxKind::Eq
            | SyntaxKind::PlusEq
            | SyntaxKind::MinusEq
            | SyntaxKind::StarEq
            | SyntaxKind::SlashEq
    ) {
        parser.bump();
        expression::expression(parser);
    }
    if parser.at(SyntaxKind::Semicolon) {
        parser.bump();
    }
    parser.finish();
}

/// Parses one braced lexical handler block and its nested statements.
fn handler_block(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::Block);
    parser.expect(SyntaxKind::LBrace, "expected `{` in handler branch");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        handler_statement(parser);
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after handler branch");
    parser.finish();
}

/// Parses a keyed model repeater.
fn repeater(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::ForExpr);
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected repeater binding");
    parser.expect(SyntaxKind::InKw, "expected `in` after repeater binding");
    expression::expression_until(parser, &[SyntaxKind::KeyKw, SyntaxKind::LBrace]);
    if parser.at(SyntaxKind::KeyKw) {
        parser.bump();
        expression::expression_until(parser, &[SyntaxKind::LBrace]);
    }
    visual_block(parser);
    parser.finish();
}

/// Parses a source-site-stable conditional and optional else branch.
fn condition(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::IfExpr);
    parser.bump();
    expression::expression_until(parser, &[SyntaxKind::LBrace]);
    visual_block(parser);
    if parser.at(SyntaxKind::ElseKw) {
        parser.start(SyntaxKind::ElseBranch);
        parser.bump();
        if parser.at(SyntaxKind::IfKw) {
            condition(parser);
        } else {
            visual_block(parser);
        }
        parser.finish();
    }
    parser.finish();
}

/// Parses a `states` block with named conditional style bodies.
fn states(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::StatesBlock);
    parser.bump();
    parser.expect(SyntaxKind::LBrace, "expected `{` after `states`");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        parser.start(SyntaxKind::StateDecl);
        parser.expect(SyntaxKind::Ident, "expected state name");
        parser.expect(SyntaxKind::WhenKw, "expected `when` after state name");
        expression::expression_until(parser, &[SyntaxKind::LBrace]);
        property_block(parser);
        parser.finish();
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after states");
    parser.finish();
}

/// Parses a property animation clause and driver body.
fn animate(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::AnimateDecl);
    parser.bump();
    parser.expect(SyntaxKind::Ident, "expected animated property name");
    parser.start(SyntaxKind::Block);
    parser.expect(
        SyntaxKind::LBrace,
        "expected `{` after animated property name",
    );
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        if parser.at(SyntaxKind::Ident)
            && parser.text() == "keyframes"
            && parser.nth_kind(1) == SyntaxKind::LBrace
        {
            keyframes(parser);
        } else if parser.at(SyntaxKind::Ident) && parser.nth_kind(1) == SyntaxKind::LBrace {
            parser.start(SyntaxKind::StyleStateDecl);
            parser.bump();
            property_block(parser);
            parser.finish();
        } else if matches!(parser.kind(), SyntaxKind::Ident | SyntaxKind::FromKw) {
            animation_assignment(parser);
        } else {
            parser.recover("expected an animation parameter or driver block");
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after animation");
    parser.finish();
    parser.finish();
}

/// Parses an animation parameter, accepting the contextual `in-out` policy.
///
/// * `parser` — active parser positioned at a parameter name.
fn animation_assignment(parser: &mut Parser<'_>) {
    if parser.text() != "transition"
        || parser.nth_kind(1) != SyntaxKind::Colon
        || parser.nth_kind(2) != SyntaxKind::InOutKw
    {
        assignment(parser);
        return;
    }
    parser.start(SyntaxKind::PropertyAssignment);
    parser.bump();
    parser.bump();
    parser.start(SyntaxKind::Expr);
    parser.start(SyntaxKind::PathExpr);
    parser.bump();
    parser.finish();
    parser.finish();
    if parser.at(SyntaxKind::Semicolon) {
        parser.bump();
    }
    parser.finish();
}

/// Parses typed value stops inside one animation timeline.
///
/// * `parser` — active parser positioned at `keyframes`.
fn keyframes(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::KeyframesDecl);
    parser.bump();
    parser.expect(SyntaxKind::LBrace, "expected `{` after `keyframes`");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        parser.start(SyntaxKind::KeyframeDecl);
        parser.expect(SyntaxKind::Number, "expected percentage keyframe offset");
        parser.expect(SyntaxKind::Colon, "expected `:` after keyframe offset");
        expression::expression(parser);
        if matches!(parser.kind(), SyntaxKind::Comma | SyntaxKind::Semicolon) {
            parser.bump();
        }
        parser.finish();
    }
    parser.expect(SyntaxKind::RBrace, "expected `}` after keyframes");
    parser.finish();
}

/// Parses a block containing property assignments or nested named state blocks.
pub(super) fn property_block(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::Block);
    parser.expect(SyntaxKind::LBrace, "expected `{`");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        if matches!(parser.kind(), SyntaxKind::Ident | SyntaxKind::ThemeName)
            && parser.nth_kind(1) == SyntaxKind::LBrace
        {
            parser.start(SyntaxKind::StyleStateDecl);
            parser.bump();
            property_block(parser);
            parser.finish();
        } else if matches!(
            parser.kind(),
            SyntaxKind::Ident | SyntaxKind::ThemeName | SyntaxKind::FromKw
        ) {
            assignment(parser);
        } else {
            parser.recover("expected a style property or state block");
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}`");
    parser.finish();
}

/// Parses a visual block containing elements, repeaters, conditions, and slot references.
fn visual_block(parser: &mut Parser<'_>) {
    parser.start(SyntaxKind::Block);
    parser.expect(SyntaxKind::LBrace, "expected `{`");
    while !parser.at(SyntaxKind::RBrace) && !parser.at(SyntaxKind::Eof) {
        match parser.kind() {
            SyntaxKind::Ident if is_element(parser) => element(parser),
            SyntaxKind::ForKw => repeater(parser),
            SyntaxKind::IfKw => condition(parser),
            SyntaxKind::Ident => slot_reference(parser),
            _ => parser.recover("expected visual content"),
        }
    }
    parser.expect(SyntaxKind::RBrace, "expected `}`");
    parser.finish();
}

/// Returns whether the token lookahead has an element type, optional ID, and block.
fn is_element(parser: &Parser<'_>) -> bool {
    parser.nth_kind(1) == SyntaxKind::LBrace
        || (parser.nth_kind(1) == SyntaxKind::Hash
            && parser.nth_kind(2) == SyntaxKind::Ident
            && parser.nth_kind(3) == SyntaxKind::LBrace)
}
