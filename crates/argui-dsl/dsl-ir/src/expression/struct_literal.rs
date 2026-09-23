//! Stable-ID lowering of checked user struct literals.

use super::*;

/// Lowers a semantically checked struct constructor and its named field values.
///
/// * `node` — struct literal syntax with named fields.
/// * `source` — owner and source identity of the whole literal.
/// * `context` — resolved type tables and lowering diagnostic sink.
///
/// Returns an ID-only constructor or an unknown placeholder on invariant failure.
pub(super) fn lower(
    node: &SyntaxNode,
    source: SourceInfo,
    context: &mut Context<'_>,
) -> IrExpression {
    let name = node
        .children()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_name(&path))
        .unwrap_or_default();
    let Some(symbol) = context.symbols.get(&name).copied() else {
        return invalid(node, context, format!("unresolved struct `{name}`"));
    };
    let Some(declared) = context.fields.get(&symbol).cloned() else {
        return invalid(node, context, format!("`{name}` is not a struct"));
    };
    let mut fields = Vec::new();
    for field in node
        .children()
        .filter(|child| child.kind() == SyntaxKind::StructFieldExpr)
    {
        let field_name = direct_name(&field).unwrap_or_default();
        let Some((id, _)) = declared.get(&field_name) else {
            return invalid(node, context, format!("unresolved field `{field_name}`"));
        };
        let Some(value) = field
            .children()
            .find(|child| child.kind() == SyntaxKind::Expr)
        else {
            return invalid(node, context, format!("missing value for `{field_name}`"));
        };
        fields.push((*id, super::lower(&value, context)));
    }
    let mut ids = declared.values().map(|(id, _)| *id).collect::<Vec<_>>();
    ids.sort_unstable();
    IrExpression {
        id: expression_id(&source),
        value_type: IrType::Struct {
            symbol,
            fields: ids,
        },
        kind: IrExpressionKind::Struct { symbol, fields },
        source,
    }
}
