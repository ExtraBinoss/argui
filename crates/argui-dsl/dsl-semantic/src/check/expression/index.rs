//! Safe array and model indexing checks.

use super::*;

/// Checks a safe array/model index and returns its optional item type.
/// Reports a type error for noninteger indices or noncollection bases.
pub(super) fn infer(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let mut children = node.children();
    let base = children
        .next()
        .map_or(Type::Unknown, |value| super::infer(&value, context));
    let Some(index) = children.find(|child| child.kind() == SyntaxKind::Expr) else {
        return Type::Unknown;
    };
    let actual = super::infer(&index, context);
    if !Type::Int.accepts(&actual) {
        super::type_mismatch(context, &index, &Type::Int, &actual);
    }
    match base {
        Type::Array(inner) | Type::Model(inner) => Type::Optional(inner),
        Type::Unknown => Type::Unknown,
        other => {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("indexing requires array or model, found `{other}`"),
                Span::new(context.file, node.text_range()),
            ));
            Type::Unknown
        }
    }
}
