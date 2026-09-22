//! Struct-field and observed native-property lowering.

use super::*;

/// Resolves a user-struct field read to a stable field ID.
pub(super) fn lower(
    node: &SyntaxNode,
    source: SourceInfo,
    context: &mut Context<'_>,
) -> IrExpression {
    let Some(base_node) = expression_children(node).next() else {
        return invalid(node, context, "member read has no base expression");
    };
    let member = tokens(node)
        .rfind(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
        .unwrap_or_default();
    if base_node.kind() == SyntaxKind::PathExpr
        && let Some(name) = direct_name(&base_node)
        && !context.locals.contains_key(&name)
        && !context.properties.contains_key(&name)
        && let Some(properties) = context
            .references
            .and_then(|references| references.get(&name))
    {
        let Some(property) = properties.get(&member) else {
            return invalid(
                node,
                context,
                format!("unresolved observed property `{name}.{member}`"),
            );
        };
        let (value_type, kind) = match property {
            ReferenceProperty::Observed {
                site,
                property,
                observation,
                value_type,
            } => (
                value_type.clone(),
                IrExpressionKind::ObservedRead {
                    site: *site,
                    property: *property,
                    observation: *observation,
                },
            ),
            ReferenceProperty::Child {
                site,
                property,
                value_type,
            } => (
                value_type.clone(),
                IrExpressionKind::ChildPropertyRead {
                    site: *site,
                    property: *property,
                },
            ),
        };
        return IrExpression {
            id: expression_id(&source),
            value_type,
            kind,
            source,
        };
    }
    let base = super::lower(&base_node, context);
    let IrType::Struct { symbol, .. } = &base.value_type else {
        return invalid(node, context, "member read base is not a struct");
    };
    let Some((field, value_type)) = context
        .fields
        .get(symbol)
        .and_then(|fields| fields.get(&member))
    else {
        return invalid(node, context, format!("unresolved struct field `{member}`"));
    };
    IrExpression {
        id: expression_id(&source),
        value_type: value_type.clone(),
        kind: IrExpressionKind::FieldRead {
            base: Box::new(base),
            field: *field,
        },
        source,
    }
}
