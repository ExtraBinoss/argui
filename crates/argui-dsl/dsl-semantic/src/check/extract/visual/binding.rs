//! Native reference indexing and writable two-way binding checks.

use std::collections::HashMap;

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    Definition, DefinitionKind, Diagnostic, DiagnosticCode, PropertyDefinition, PropertyDirection,
    SymbolId, Type, check::Scope, lower::direct_tokens,
};

/// Checks that `value` names a writable member of `properties` with `expected` type.
/// `updates` indicates that the child/native can publish changes; errors are
/// appended to `diagnostics` at the expression's span in `file`.
pub(super) fn validate_two_way(
    value: &SyntaxNode,
    file: FileId,
    properties: &HashMap<String, PropertyDefinition>,
    expected: &Type,
    updates: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let target = crate::check::statement::property_name(value);
    let writable = target
        .as_ref()
        .and_then(|name| properties.get(name))
        .is_some_and(|property| {
            matches!(
                property.direction,
                PropertyDirection::Private
                    | PropertyDirection::Output
                    | PropertyDirection::InputOutput
            )
        });
    let same_type = target
        .as_ref()
        .and_then(|name| properties.get(name))
        .is_none_or(|property| &property.value_type == expected);
    if !writable || !same_type || !updates {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidTwoWayBinding,
            "two-way binding requires a writable local property of the same type and a child property that publishes changes",
            Span::new(file, value.text_range()),
        ));
    }
}

/// Indexes explicitly identified native and child-component outputs.
///
/// * `syntax` — owning component declaration.
/// * `scope` — native imports visible in the source module.
/// * `schema` — registry defining observable native outputs.
/// * `definitions` — resolved component declarations and their output types.
/// * `file` — source file used for duplicate identity diagnostics.
/// * `diagnostics` — output receiving duplicate identity errors.
///
/// Returns readable properties by source identity. Repeated elements are excluded
/// because one name would otherwise identify multiple mounted values.
pub(super) fn native_references(
    syntax: &SyntaxNode,
    scope: &Scope,
    schema: &argui_schema::SchemaRegistry,
    definitions: &HashMap<SymbolId, Definition>,
    file: FileId,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, HashMap<String, Type>> {
    let mut references = HashMap::new();
    for element in syntax
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::Element)
    {
        let Some(id) = element
            .children()
            .find(|child| child.kind() == SyntaxKind::ElementId)
            .and_then(|node| direct_ident(&node))
        else {
            continue;
        };
        if element
            .ancestors()
            .any(|ancestor| ancestor.kind() == SyntaxKind::ForExpr)
        {
            continue;
        }
        let Some(element_name) = direct_ident(&element) else {
            continue;
        };
        let properties: HashMap<String, Type> = if let Some(native) = scope
            .natives
            .get(&element_name)
            .and_then(|id| schema.schema(*id))
        {
            native
                .properties
                .iter()
                .filter_map(|property| {
                    property.observation.map(|observation| {
                        let value_type = match observation {
                            argui_schema::ObservationKind::Hover
                            | argui_schema::ObservationKind::Pressed
                            | argui_schema::ObservationKind::Focused
                            | argui_schema::ObservationKind::FocusVisible => Type::Bool,
                            _ => Type::Length,
                        };
                        (property.name.as_str().to_string(), value_type)
                    })
                })
                .collect()
        } else if let Some(Definition {
            kind: DefinitionKind::Component(component),
            ..
        }) = scope
            .symbols
            .get(&element_name)
            .and_then(|id| definitions.get(id))
        {
            if element
                .ancestors()
                .any(|ancestor| ancestor.kind() == SyntaxKind::IfExpr)
            {
                continue;
            }
            component
                .properties
                .iter()
                .filter(|property| {
                    matches!(
                        property.direction,
                        PropertyDirection::Output | PropertyDirection::InputOutput
                    )
                })
                .map(|property| (property.name.clone(), property.value_type.clone()))
                .collect()
        } else {
            continue;
        };
        if references.insert(id.clone(), properties).is_some() {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateMember,
                format!("element identity `#{id}` is declared more than once"),
                Span::new(file, element.text_range()),
            ));
        }
    }
    references
}

/// Returns the first direct identifier in one syntax node.
///
/// * `node` — syntax node whose own token identifies a target or source ID.
///
/// Returns the identifier text when present.
fn direct_ident(node: &SyntaxNode) -> Option<String> {
    direct_tokens(node)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}
