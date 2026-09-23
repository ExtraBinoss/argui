//! Struct-field and observed native-property reads.

use super::*;

/// Resolves a struct member after inferring its base expression.
pub(super) fn infer(node: &SyntaxNode, context: &mut Context<'_, '_>) -> Type {
    let base_node = node
        .children()
        .find(|child| is_expression_kind(child.kind()));
    let member = node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::Ident)
        .last()
        .map(|token| token.text().to_string());
    let Some(member) = member else {
        return Type::Unknown;
    };
    if let Some(name) = base_node
        .as_ref()
        .filter(|base| base.kind() == SyntaxKind::PathExpr)
        .and_then(direct_ident)
        && !context.locals.contains_key(&name)
        && !context.properties.contains_key(&name)
        && let Some(properties) = context
            .references
            .and_then(|references| references.get(&name))
    {
        return properties.get(&member).cloned().unwrap_or_else(|| {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownProperty,
                format!("`{name}` has no readable property `{member}`"),
                Span::new(context.file, node.text_range()),
            ));
            Type::Unknown
        });
    }
    if let Some(name) = base_node
        .as_ref()
        .filter(|base| base.kind() == SyntaxKind::PathExpr)
        .and_then(direct_ident)
        && !context.locals.contains_key(&name)
        && !context.properties.contains_key(&name)
        && let Some(symbol) = context.symbols.get(&name).copied()
        && let Some(Definition {
            kind: DefinitionKind::Enum(definition),
            ..
        }) = context.definitions.get(&symbol)
    {
        if definition
            .variants
            .iter()
            .any(|(variant, _)| variant == &member)
        {
            return Type::Enum(symbol);
        }
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnknownName,
            format!("enum `{name}` has no variant `{member}`"),
            Span::new(context.file, node.text_range()),
        ));
        return Type::Unknown;
    }
    let base = base_node.map_or(Type::Unknown, |child| super::infer(&child, context));
    let Type::Struct(id) = base else {
        if base != Type::Unknown {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("type `{base}` has no member `{member}`"),
                Span::new(context.file, node.text_range()),
            ));
        }
        return Type::Unknown;
    };
    let Some(Definition {
        kind: DefinitionKind::Struct(definition),
        ..
    }) = context.definitions.get(&id)
    else {
        return Type::Unknown;
    };
    if let Some(field) = definition.fields.iter().find(|field| field.name == member) {
        field.value_type.clone()
    } else {
        context.diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnknownName,
            format!("struct has no field `{member}`"),
            Span::new(context.file, node.text_range()),
        ));
        Type::Unknown
    }
}
