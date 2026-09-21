use std::collections::BTreeMap;

use argui_dsl_syntax::{SyntaxKind, SyntaxNode};

use crate::{CompilerDatabase, DefinitionKind, Type, types::from_schema};

use super::{Completion, SourceContext, SymbolKind, ToolingError, direct_name};

/// Produces completion candidates from the checked project and canonical schema.
pub(super) fn completions(
    database: &mut CompilerDatabase,
    path: &str,
    offset: u32,
) -> Result<Vec<Completion>, ToolingError> {
    let context = database.source_context(path, offset)?;
    let source = database.source(context.file).unwrap_or_default();
    let prefix = &source[..usize::try_from(offset).unwrap_or(source.len())];
    let mut output = BTreeMap::new();
    if prefix
        .rsplit_once("var(")
        .is_some_and(|(_, suffix)| !suffix.contains(')') && suffix.trim_start().starts_with("--"))
    {
        complete_theme_tokens(
            &context,
            expected_property_type(database, &context),
            &mut output,
        );
    } else if let Some(import) = ancestor(&context, SyntaxKind::ImportList) {
        let _ = import;
        complete_imports(database, &context, &mut output);
    } else if let Some(element) = ancestor(&context, SyntaxKind::Element) {
        complete_element(database, &context, &element, &mut output);
    } else {
        complete_scope(database, &context, &mut output);
    }
    Ok(output.into_values().collect())
}

/// Returns the nearest ancestor with one syntax kind.
fn ancestor(context: &SourceContext, kind: SyntaxKind) -> Option<SyntaxNode> {
    context
        .token
        .as_ref()?
        .parent_ancestors()
        .find(|node| {
            node.kind() == kind || node.parent().is_some_and(|parent| parent.kind() == kind)
        })
        .and_then(|node| {
            if node.kind() == kind {
                Some(node)
            } else {
                node.parent()
            }
        })
}

/// Completes public standard-library components and low-level native primitives.
fn complete_imports(
    database: &CompilerDatabase,
    context: &SourceContext,
    output: &mut BTreeMap<String, Completion>,
) {
    if let Some(module) = context
        .project
        .modules
        .iter()
        .find(|module| module.path == "@argui/ui")
    {
        for definition in module
            .definitions
            .iter()
            .filter(|definition| definition.exported)
        {
            insert_definition(output, definition);
        }
    }
    for schema in database.schema().schemas() {
        output.entry(schema.name.to_string()).or_insert(Completion {
            label: schema.name.to_string(),
            kind: SymbolKind::Native,
            detail: "native primitive".into(),
            documentation: schema.documentation.clone(),
        });
    }
}

/// Completes properties and events valid for the containing element target.
fn complete_element(
    database: &CompilerDatabase,
    context: &SourceContext,
    element: &SyntaxNode,
    output: &mut BTreeMap<String, Completion>,
) {
    let Some(target) = direct_name(element) else {
        return;
    };
    let assigned = element
        .children()
        .filter(|child| {
            matches!(
                child.kind(),
                SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
            )
        })
        .filter_map(|child| direct_name(&child))
        .collect::<Vec<_>>();
    if let Some(native) = context
        .module
        .native_scope
        .get(&target)
        .and_then(|id| database.schema().schema(*id))
    {
        for property in &native.properties {
            if !assigned.iter().any(|name| name == property.name.as_str()) {
                output.insert(
                    property.name.to_string(),
                    Completion {
                        label: property.name.to_string(),
                        kind: SymbolKind::Property,
                        detail: format!("{:?}", property.value_type),
                        documentation: property.documentation.clone(),
                    },
                );
            }
        }
        for event in &native.events {
            output.insert(
                format!("on {}", event.name),
                Completion {
                    label: format!("on {}", event.name),
                    kind: SymbolKind::Callback,
                    detail: event
                        .payload
                        .map_or_else(|| "event".into(), |value| format!("event<{value:?}>")),
                    documentation: event.documentation.clone(),
                },
            );
        }
        return;
    }
    let Some(component) = context
        .module
        .scope
        .get(&target)
        .and_then(|id| context.project.definition(*id))
        .and_then(|definition| match &definition.kind {
            DefinitionKind::Component(component) => Some(component),
            _ => None,
        })
    else {
        return;
    };
    for property in &component.properties {
        if !assigned.contains(&property.name) {
            output.insert(
                property.name.clone(),
                Completion {
                    label: property.name.clone(),
                    kind: SymbolKind::Property,
                    detail: property.value_type.to_string(),
                    documentation: format!("{:?} component property", property.direction),
                },
            );
        }
    }
    for callback in &component.callbacks {
        output.insert(
            format!("on {}", callback.name),
            Completion {
                label: format!("on {}", callback.name),
                kind: SymbolKind::Callback,
                detail: callback_signature(callback),
                documentation: "Component callback handler.".into(),
            },
        );
    }
}

/// Completes compatible theme tokens for the expected property type.
fn complete_theme_tokens(
    context: &SourceContext,
    expected: Option<Type>,
    output: &mut BTreeMap<String, Completion>,
) {
    for definition in context
        .project
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
    {
        let DefinitionKind::Theme(theme) = &definition.kind else {
            continue;
        };
        for token in &theme.tokens {
            if expected
                .as_ref()
                .is_none_or(|expected| expected.accepts(&token.value_type))
            {
                output.insert(
                    token.name.clone(),
                    Completion {
                        label: token.name.clone(),
                        kind: SymbolKind::Token,
                        detail: token.value_type.to_string(),
                        documentation: format!("Theme token from `{}`.", definition.name),
                    },
                );
            }
        }
    }
}

/// Resolves the expected type of the containing native property assignment.
fn expected_property_type(database: &CompilerDatabase, context: &SourceContext) -> Option<Type> {
    let assignment = ancestor(context, SyntaxKind::PropertyAssignment)?;
    let property = direct_name(&assignment)?;
    let element = assignment
        .ancestors()
        .skip(1)
        .find(|node| node.kind() == SyntaxKind::Element)?;
    let target = direct_name(&element)?;
    if let Some(schema) = context
        .module
        .native_scope
        .get(&target)
        .and_then(|id| database.schema().schema(*id))
    {
        return schema
            .properties
            .iter()
            .find(|candidate| candidate.name.as_str() == property)
            .map(|candidate| from_schema(candidate.value_type));
    }
    context
        .module
        .scope
        .get(&target)
        .and_then(|id| context.project.definition(*id))
        .and_then(|definition| match &definition.kind {
            DefinitionKind::Component(component) => component
                .properties
                .iter()
                .find(|candidate| candidate.name == property)
                .map(|candidate| candidate.value_type.clone()),
            _ => None,
        })
}

/// Completes visible definitions plus declaration keywords outside elements.
fn complete_scope(
    database: &CompilerDatabase,
    context: &SourceContext,
    output: &mut BTreeMap<String, Completion>,
) {
    for keyword in [
        "import",
        "export component",
        "export theme",
        "export style",
        "export effect",
    ] {
        output.insert(
            keyword.into(),
            Completion {
                label: keyword.into(),
                kind: SymbolKind::Keyword,
                detail: "declaration".into(),
                documentation: "Argui DSL declaration.".into(),
            },
        );
    }
    for id in context.module.scope.values() {
        if let Some(definition) = context.project.definition(*id) {
            insert_definition(output, definition);
        }
    }
    complete_imports(database, context, output);
}

/// Inserts one semantic definition using its canonical completion metadata.
fn insert_definition(output: &mut BTreeMap<String, Completion>, definition: &crate::Definition) {
    let (kind, detail) = match &definition.kind {
        DefinitionKind::Component(_) => (SymbolKind::Component, "component"),
        DefinitionKind::Struct(_) => (SymbolKind::Struct, "struct"),
        DefinitionKind::Enum(_) => (SymbolKind::Enum, "enum"),
        DefinitionKind::Theme(_) => (SymbolKind::Theme, "theme"),
        DefinitionKind::Style(_) => (SymbolKind::Style, "style"),
        DefinitionKind::Effect(_) => (SymbolKind::Effect, "effect"),
    };
    output.insert(
        definition.name.clone(),
        Completion {
            label: definition.name.clone(),
            kind,
            detail: detail.into(),
            documentation: format!("Argui {detail} `{}`.", definition.name),
        },
    );
}

/// Formats one callback's typed completion detail.
fn callback_signature(callback: &crate::CallbackDefinition) -> String {
    let parameters = callback
        .parameters
        .iter()
        .map(|parameter| format!("{}: {}", parameter.name, parameter.value_type))
        .collect::<Vec<_>>()
        .join(", ");
    format!("callback({parameters}) -> {}", callback.result)
}
