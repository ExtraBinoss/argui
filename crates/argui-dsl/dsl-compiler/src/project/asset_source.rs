//! Source locations of assets retained through declarative expressions.

use argui_dsl_ir::{AssetId, IrExpression, IrExpressionKind, IrNode, IrProject, IrStatement};
use argui_dsl_syntax::Span;

/// Finds one source expression that references an asset ID.
///
/// * `ir` — lowered project containing asset uses.
/// * `id` — canonical asset identity reported by the validator.
///
/// Returns the expression span, or an effect declaration span for shader assets.
pub(super) fn find(ir: &IrProject, id: AssetId) -> Option<Span> {
    ir.components
        .iter()
        .find_map(|component| {
            component
                .properties
                .iter()
                .filter_map(|property| property.default.as_ref())
                .find_map(|value| expression(value, id))
                .or_else(|| component.body.iter().find_map(|node| visual_node(node, id)))
                .or_else(|| {
                    component.states.iter().find_map(|state| {
                        expression(&state.condition, id).or_else(|| {
                            state
                                .assignments
                                .iter()
                                .find_map(|(_, value)| expression(value, id))
                        })
                    })
                })
                .or_else(|| {
                    component.animations.iter().find_map(|animation| {
                        animation
                            .parameters
                            .iter()
                            .find_map(|parameter| expression(&parameter.value, id))
                    })
                })
        })
        .or_else(|| {
            ir.themes.iter().find_map(|theme| {
                theme
                    .tokens
                    .iter()
                    .find_map(|token| expression(&token.default, id))
                    .or_else(|| {
                        theme.modes.iter().find_map(|mode| {
                            mode.overrides
                                .iter()
                                .find_map(|(_, value)| expression(value, id))
                        })
                    })
            })
        })
        .or_else(|| {
            ir.styles.iter().find_map(|style| {
                style
                    .properties
                    .iter()
                    .find_map(|binding| expression(&binding.value, id))
                    .or_else(|| {
                        style.states.iter().find_map(|state| {
                            state
                                .properties
                                .iter()
                                .find_map(|binding| expression(&binding.value, id))
                        })
                    })
            })
        })
        .or_else(|| {
            ir.effects.iter().find_map(|effect| {
                (effect.shader == id)
                    .then_some(effect.source.span)
                    .flatten()
                    .or_else(|| {
                        effect
                            .parameters
                            .iter()
                            .filter_map(|parameter| parameter.default.as_ref())
                            .find_map(|value| expression(value, id))
                    })
            })
        })
}

/// Searches a visual subtree and its handler expressions for an asset use.
///
/// * `node` — current visual tree node.
/// * `id` — canonical asset identity.
///
/// Returns the first matching source span.
fn visual_node(node: &IrNode, id: AssetId) -> Option<Span> {
    match node {
        IrNode::Element {
            properties,
            effect,
            events,
            children,
            ..
        } => properties
            .iter()
            .find_map(|binding| expression(&binding.value, id))
            .or_else(|| {
                effect.as_ref().and_then(|binding| {
                    binding
                        .parameters
                        .iter()
                        .find_map(|parameter| expression(&parameter.value, id))
                })
            })
            .or_else(|| {
                events.iter().find_map(|event| {
                    event
                        .statements
                        .iter()
                        .find_map(|statement| event_statement(statement, id))
                })
            })
            .or_else(|| children.iter().find_map(|child| visual_node(child, id))),
        IrNode::Repeater {
            model, key, body, ..
        } => expression(model, id)
            .or_else(|| expression(key, id))
            .or_else(|| body.iter().find_map(|child| visual_node(child, id))),
        IrNode::Conditional {
            condition,
            then_body,
            else_body,
            ..
        } => expression(condition, id).or_else(|| {
            then_body
                .iter()
                .chain(else_body)
                .find_map(|child| visual_node(child, id))
        }),
        IrNode::Slot { fallback: body, .. } | IrNode::SlotContent { body, .. } => {
            body.iter().find_map(|child| visual_node(child, id))
        }
    }
}

/// Searches one event statement's value expression for an asset use.
///
/// * `statement` — lowered handler instruction.
/// * `id` — canonical asset identity.
///
/// Returns the matching source span, if present.
fn event_statement(statement: &IrStatement, id: AssetId) -> Option<Span> {
    match statement {
        IrStatement::Expression(value)
        | IrStatement::SetThemeMode(value)
        | IrStatement::Assignment { value, .. }
        | IrStatement::Return(Some(value)) => expression(value, id),
        IrStatement::ScrollTo { x, y, .. } => expression(x, id).or_else(|| expression(y, id)),
        IrStatement::Return(None)
        | IrStatement::FocusNext
        | IrStatement::FocusPrevious
        | IrStatement::PreventDefault
        | IrStatement::StopPropagation => None,
    }
}

/// Searches a typed expression and every nested operand for an asset use.
///
/// * `value` — current typed expression.
/// * `id` — canonical asset identity.
///
/// Returns the matching expression's source span, if present.
fn expression(value: &IrExpression, id: AssetId) -> Option<Span> {
    match &value.kind {
        IrExpressionKind::Asset(asset) if *asset == id => value.source.span,
        IrExpressionKind::FieldRead { base, .. }
        | IrExpressionKind::Unary { operand: base, .. } => expression(base, id),
        IrExpressionKind::BuiltinCall { arguments, .. }
        | IrExpressionKind::CallbackCall { arguments, .. }
        | IrExpressionKind::Array(arguments) => arguments
            .iter()
            .find_map(|argument| expression(argument, id)),
        IrExpressionKind::Binary { left, right, .. } => {
            expression(left, id).or_else(|| expression(right, id))
        }
        IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => expression(condition, id)
            .or_else(|| expression(then_value, id))
            .or_else(|| expression(else_value, id)),
        IrExpressionKind::Constant(_)
        | IrExpressionKind::PropertyRead(_)
        | IrExpressionKind::ChildPropertyRead { .. }
        | IrExpressionKind::ObservedRead { .. }
        | IrExpressionKind::LocalRead(_)
        | IrExpressionKind::TokenRead(_)
        | IrExpressionKind::Asset(_) => None,
    }
}
