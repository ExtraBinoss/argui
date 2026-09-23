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
/// `scoped` allows references inside the given repeater only; nested repeaters are
/// excluded. Returns readable properties by source identity in this lexical scope.
pub(super) fn native_references(
    syntax: &SyntaxNode,
    scope: &Scope,
    schema: &argui_schema::SchemaRegistry,
    definitions: &HashMap<SymbolId, Definition>,
    file: FileId,
    diagnostics: &mut Vec<Diagnostic>,
    scoped: bool,
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
            .take_while(|ancestor| ancestor != syntax)
            .any(|ancestor| ancestor.kind() == SyntaxKind::ForExpr)
        {
            continue;
        }
        if !scoped && syntax.kind() == SyntaxKind::ForExpr {
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
            let conditional = element
                .ancestors()
                .take_while(|ancestor| ancestor != syntax)
                .any(|ancestor| ancestor.kind() == SyntaxKind::IfExpr);
            component
                .properties
                .iter()
                .filter(|property| {
                    matches!(
                        property.direction,
                        PropertyDirection::Output | PropertyDirection::InputOutput
                    )
                })
                .map(|property| {
                    let value_type = if conditional {
                        Type::Optional(Box::new(property.value_type.clone()))
                    } else {
                        property.value_type.clone()
                    };
                    (property.name.clone(), value_type)
                })
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

/// Reports whether `value` reads a completed-layout width or height.
///
/// `value` is a checked binding expression. The result is true only for member
/// reads, so string literals mentioning the property do not trigger the guard.
pub(crate) fn reads_measured_bounds(value: &SyntaxNode) -> bool {
    value
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::MemberExpr)
        .any(|node| {
            direct_tokens(&node).any(|token| {
                token.kind() == SyntaxKind::Ident
                    && matches!(token.text(), "measured_width" | "measured_height")
            })
        })
}

/// Whether one native property can change measured layout geometry.
///
/// `name` is the native assignment target; the result conservatively includes
/// size, positioning, spacing, grid, and flex controls.
pub(super) fn drives_layout(name: &str) -> bool {
    matches!(
        name,
        "width"
            | "height"
            | "min_width"
            | "min_height"
            | "max_width"
            | "max_height"
            | "aspect_ratio"
            | "flex_basis"
            | "grow"
            | "shrink"
            | "wrap"
            | "x"
            | "y"
            | "left"
            | "right"
            | "top"
            | "bottom"
            | "padding"
            | "padding_left"
            | "padding_right"
            | "padding_top"
            | "padding_bottom"
            | "margin"
            | "margin_left"
            | "margin_right"
            | "margin_top"
            | "margin_bottom"
            | "gap"
            | "row_gap"
            | "column_gap"
            | "grid_columns"
            | "grid_rows"
            | "grid_column"
            | "grid_row"
            | "grid_column_span"
            | "grid_row_span"
    )
}
