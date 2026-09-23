//! Validation of keyed repeaters and their lexical child scope.

use super::*;

/// Checks the collection and key types for `node`, then validates its rows with
/// `locals` and row-local child references. `file` locates diagnostics; `scope`,
/// `definitions`, `theme_tokens`, `schema`, `references`, `component_properties`
/// and `callbacks` provide the surrounding DSL contract. Errors are appended to
/// `diagnostics`.
#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    node: &SyntaxNode,
    file: FileId,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    references: &HashMap<String, HashMap<String, Type>>,
    component_properties: &HashMap<String, PropertyDefinition>,
    callbacks: &HashMap<String, CallbackDefinition>,
    locals: &HashMap<String, Type>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let collection = node
        .children()
        .find(|child| child.kind() == SyntaxKind::Expr);
    let mut next_locals = locals.clone();
    if let Some(collection) = collection {
        let mut context = expression::Context {
            symbols: scope.symbols.clone(),
            file,
            properties: component_properties,
            callbacks,
            locals: locals.clone(),
            definitions,
            theme_tokens,
            references: Some(references),
            event_handler: false,
            diagnostics,
        };
        let value = expression::infer(&collection, &mut context);
        let item = match value {
            Type::Model(item) | Type::Array(item) => *item,
            Type::Unknown => Type::Unknown,
            other => {
                context.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("repeater source must be model<T> or array<T>, found `{other}`"),
                    Span::new(file, collection.text_range()),
                ));
                Type::Unknown
            }
        };
        if let Some(binding) = identifier_after(node, SyntaxKind::ForKw) {
            next_locals.insert(binding, item);
        }
    }
    if !has_direct_token(node, SyntaxKind::KeyKw) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::MissingRepeaterKey,
            "repeater requires a stable `key` expression of type int or string",
            Span::new(file, node.text_range()),
        ));
    } else if let Some(key) = node
        .children()
        .filter(|child| child.kind() == SyntaxKind::Expr)
        .nth(1)
    {
        let mut context = expression::Context {
            symbols: scope.symbols.clone(),
            file,
            properties: component_properties,
            callbacks,
            locals: next_locals.clone(),
            definitions,
            theme_tokens,
            references: Some(references),
            event_handler: false,
            diagnostics,
        };
        let actual = expression::infer(&key, &mut context);
        if !matches!(actual, Type::Int | Type::String | Type::Unknown) {
            context.diagnostics.push(Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "repeater key must be int or string",
                Span::new(file, key.text_range()),
            ));
        }
    }
    let mut nested_references = references.clone();
    nested_references.extend(binding::native_references(
        node,
        scope,
        schema,
        definitions,
        file,
        diagnostics,
        true,
    ));
    for child in visual_children(node) {
        validate_visual(
            &child,
            file,
            scope,
            definitions,
            theme_tokens,
            schema,
            &nested_references,
            component_properties,
            callbacks,
            &next_locals,
            diagnostics,
        );
    }
}
