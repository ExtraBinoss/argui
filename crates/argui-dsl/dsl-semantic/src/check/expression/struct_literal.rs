//! Typed construction of user structs in expression position.

use std::collections::HashSet;

use super::*;

/// Checks a named struct literal against its declared fields.
/// Reports duplicate, unknown, missing, and incorrectly typed fields at source spans.
pub(super) fn infer(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let name = node
        .children()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_ident(&path))
        .unwrap_or_default();
    let Some(symbol) = context.symbols.get(&name).copied() else {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnknownName,
            format!("unknown struct `{name}`"),
            Span::new(context.file, node.text_range()),
        ));
        return Type::Unknown;
    };
    let Some(Definition {
        kind: DefinitionKind::Struct(structure),
        ..
    }) = context.definitions.get(&symbol)
    else {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::TypeMismatch,
            format!("`{name}` is not a struct"),
            Span::new(context.file, node.text_range()),
        ));
        return Type::Unknown;
    };
    let fields = structure.fields.clone();
    let mut seen = HashSet::new();
    for field in node
        .children()
        .filter(|child| child.kind() == SyntaxKind::StructFieldExpr)
    {
        let field_name = direct_ident(&field).unwrap_or_default();
        if !seen.insert(field_name.clone()) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateMember,
                format!("struct field `{field_name}` occurs more than once"),
                Span::new(context.file, field.text_range()),
            ));
        }
        let expected = fields.iter().find(|candidate| candidate.name == field_name);
        if expected.is_none() {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownName,
                format!("unknown field `{field_name}` on `{name}`"),
                Span::new(context.file, field.text_range()),
            ));
        }
        if let Some(value) = field
            .children()
            .find(|child| child.kind() == SyntaxKind::Expr)
        {
            let actual = super::infer(&value, context);
            if let Some(expected) = expected
                && !expected.value_type.accepts(&actual)
            {
                super::type_mismatch(context, &value, &expected.value_type, &actual);
            }
        }
    }
    for field in &fields {
        if !seen.contains(&field.name) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("missing field `{}` in `{name}` literal", field.name),
                Span::new(context.file, node.text_range()),
            ));
        }
    }
    Type::Struct(symbol)
}
