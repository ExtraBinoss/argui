use argui_dsl_syntax::{Span, SyntaxKind, SyntaxToken};

use crate::{CompilerDatabase, DefinitionKind, PropertyDirection, SymbolId};

use super::{
    Location, RenameEdit, SourceContext, Symbol, SymbolKind, ToolingError, contains, direct_name,
};

/// Semantic identity resolved at one source token.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Key {
    Definition(SymbolId),
    Member {
        component: SymbolId,
        name: String,
        kind: SymbolKind,
    },
    ThemeToken(String),
}

/// Returns Markdown hover text for a resolved symbol or schema member.
pub(super) fn hover(
    database: &mut CompilerDatabase,
    path: &str,
    offset: u32,
) -> Result<Option<String>, ToolingError> {
    let context = database.source_context(path, offset)?;
    let Some(token) = &context.token else {
        return Ok(None);
    };
    if let Some(key) = resolve_key(&context, token, offset) {
        return Ok(Some(describe_key(&context, &key)));
    }
    let name = token.text();
    if let Some(schema) = context
        .module
        .native_scope
        .get(name)
        .and_then(|id| database.schema().schema(*id))
    {
        return Ok(Some(format!(
            "**native {}**\n\n{}",
            schema.name, schema.documentation
        )));
    }
    if let Some(description) = element_member_hover(database, &context, token) {
        return Ok(Some(description));
    }
    Ok(None)
}

/// Resolves the declaration location of a symbol under the cursor.
pub(super) fn definition(
    database: &mut CompilerDatabase,
    path: &str,
    offset: u32,
) -> Result<Option<Location>, ToolingError> {
    let context = database.source_context(path, offset)?;
    let Some(token) = &context.token else {
        return Ok(None);
    };
    Ok(resolve_key(&context, token, offset)
        .and_then(|key| declaration_span(&context, &key))
        .and_then(|span| database.location(span)))
}

/// Finds every checked source reference to one semantic identity.
pub(super) fn references(
    database: &mut CompilerDatabase,
    path: &str,
    offset: u32,
) -> Result<Vec<Location>, ToolingError> {
    let context = database.source_context(path, offset)?;
    let Some(token) = &context.token else {
        return Ok(Vec::new());
    };
    let Some(key) = resolve_key(&context, token, offset) else {
        return Ok(Vec::new());
    };
    Ok(reference_locations(database, &context, &key))
}

/// Validates a replacement and maps semantic references to workspace edits.
pub(super) fn rename(
    database: &mut CompilerDatabase,
    path: &str,
    offset: u32,
    replacement: &str,
) -> Result<Vec<RenameEdit>, ToolingError> {
    if !valid_identifier(replacement) {
        return Err(ToolingError::InvalidIdentifier(replacement.into()));
    }
    references(database, path, offset).map(|locations| {
        locations
            .into_iter()
            .map(|location| RenameEdit {
                location,
                replacement: replacement.into(),
            })
            .collect()
    })
}

/// Returns stable source symbols for a document or the user workspace.
pub(super) fn symbols(
    database: &mut CompilerDatabase,
    selected: Option<&str>,
) -> Result<Vec<Symbol>, ToolingError> {
    if let Some(path) = selected
        && database.file_id(path).is_none()
    {
        return Err(ToolingError::UnknownModule(path.into()));
    }
    let project = database.check();
    let mut symbols = project
        .modules
        .iter()
        .filter(|module| {
            selected.map_or_else(|| !module.path.starts_with('@'), |path| module.path == path)
        })
        .flat_map(|module| &module.definitions)
        .filter_map(|definition| {
            Some(Symbol {
                name: definition.name.clone(),
                kind: definition_kind(&definition.kind),
                location: database.location(definition.span)?,
            })
        })
        .collect::<Vec<_>>();
    symbols.sort_by(|left, right| {
        left.location
            .path
            .cmp(&right.location.path)
            .then(left.location.start.cmp(&right.location.start))
    });
    Ok(symbols)
}

/// Resolves a top-level definition, component member, or theme token.
fn resolve_key(context: &SourceContext, token: &SyntaxToken, offset: u32) -> Option<Key> {
    let name = token.text().to_owned();
    if token.kind() == SyntaxKind::ThemeName {
        return Some(Key::ThemeToken(name));
    }
    if let Some(component) = containing_component(context, offset)
        && let DefinitionKind::Component(value) = &component.kind
    {
        if value.properties.iter().any(|member| member.name == name)
            && is_member_reference(
                token,
                &value.properties.iter().find(|m| m.name == name)?.span,
            )
        {
            return Some(Key::Member {
                component: component.id,
                name,
                kind: SymbolKind::Property,
            });
        }
        if value.callbacks.iter().any(|member| member.name == name) {
            return Some(Key::Member {
                component: component.id,
                name,
                kind: SymbolKind::Callback,
            });
        }
        if value.slots.iter().any(|member| member.name == name) {
            return Some(Key::Member {
                component: component.id,
                name,
                kind: SymbolKind::Slot,
            });
        }
    }
    context
        .module
        .scope
        .get(&name)
        .copied()
        .map(Key::Definition)
}

/// Rejects an element-property destination while accepting reads and declarations.
fn is_member_reference(token: &SyntaxToken, declaration: &Span) -> bool {
    if declaration.range.contains_range(token.text_range()) {
        return true;
    }
    !token.parent_ancestors().any(|node| {
        matches!(
            node.kind(),
            SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
        ) && direct_name(&node).as_deref() == Some(token.text())
            && node
                .children_with_tokens()
                .filter_map(|item| item.into_token())
                .find(|candidate| candidate.kind() == SyntaxKind::Ident)
                .is_some_and(|candidate| candidate.text_range() == token.text_range())
    })
}

/// Finds the semantic component whose declaration encloses an offset.
fn containing_component(context: &SourceContext, offset: u32) -> Option<&crate::Definition> {
    context.module.definitions.iter().find(|definition| {
        matches!(definition.kind, DefinitionKind::Component(_))
            && contains(definition.span.range, offset)
    })
}

/// Returns the declaration span represented by one semantic key.
fn declaration_span(context: &SourceContext, key: &Key) -> Option<Span> {
    match key {
        Key::Definition(id) => context.project.definition(*id).map(|value| value.span),
        Key::Member {
            component,
            name,
            kind,
        } => {
            let definition = context.project.definition(*component)?;
            let DefinitionKind::Component(component) = &definition.kind else {
                return None;
            };
            match kind {
                SymbolKind::Property => component
                    .properties
                    .iter()
                    .find(|member| member.name == *name)
                    .map(|member| member.span),
                SymbolKind::Callback => component
                    .callbacks
                    .iter()
                    .find(|member| member.name == *name)
                    .map(|member| member.span),
                SymbolKind::Slot => component
                    .slots
                    .iter()
                    .find(|member| member.name == *name)
                    .map(|member| member.span),
                _ => None,
            }
        }
        Key::ThemeToken(name) => context
            .project
            .modules
            .iter()
            .flat_map(|module| &module.definitions)
            .filter_map(|definition| match &definition.kind {
                DefinitionKind::Theme(theme) => Some(theme),
                _ => None,
            })
            .flat_map(|theme| &theme.tokens)
            .find(|token| token.name == *name)
            .map(|token| token.span),
    }
}

/// Describes one resolved key with its type and data-flow contract.
fn describe_key(context: &SourceContext, key: &Key) -> String {
    match key {
        Key::Definition(id) => {
            let Some(definition) = context.project.definition(*id) else {
                return "Unresolved definition.".into();
            };
            format!(
                "**{} {}**\n\nSource-defined Argui {}.",
                kind_label(definition_kind(&definition.kind)),
                definition.name,
                kind_label(definition_kind(&definition.kind))
            )
        }
        Key::Member {
            component,
            name,
            kind,
        } => {
            let Some(definition) = context.project.definition(*component) else {
                return "Unresolved member.".into();
            };
            let DefinitionKind::Component(component) = &definition.kind else {
                return "Unresolved member.".into();
            };
            match kind {
                SymbolKind::Property => component
                    .properties
                    .iter()
                    .find(|property| property.name == *name)
                    .map_or_else(
                        || "Unresolved property.".into(),
                        |property| {
                            format!(
                                "**property {name}: {}**\n\n{} data flow.",
                                property.value_type,
                                direction_label(property.direction)
                            )
                        },
                    ),
                SymbolKind::Callback => format!("**callback {name}**\n\nComponent event contract."),
                SymbolKind::Slot => format!("**slot {name}**\n\nVisual child-content slot."),
                _ => "Unresolved member.".into(),
            }
        }
        Key::ThemeToken(name) => format!("**theme token {name}**"),
    }
}

/// Describes a native or component property/event under the cursor.
fn element_member_hover(
    database: &CompilerDatabase,
    context: &SourceContext,
    token: &SyntaxToken,
) -> Option<String> {
    let member = token.text();
    let element = token
        .parent_ancestors()
        .find(|node| node.kind() == SyntaxKind::Element)?;
    let target = direct_name(&element)?;
    let schema = context
        .module
        .native_scope
        .get(&target)
        .and_then(|id| database.schema().schema(*id))?;
    if let Some(property) = schema
        .properties
        .iter()
        .find(|property| property.name.as_str() == member)
    {
        return Some(format!(
            "**{}.{member}: {:?}**\n\n{}",
            schema.name, property.value_type, property.documentation
        ));
    }
    schema
        .events
        .iter()
        .find(|event| event.name.as_str() == member)
        .map(|event| format!("**{}.on {member}**\n\n{}", schema.name, event.documentation))
}

/// Collects every token whose semantic identity equals `key`.
fn reference_locations(
    database: &CompilerDatabase,
    context: &SourceContext,
    key: &Key,
) -> Vec<Location> {
    let mut output = Vec::new();
    let member_declaration = declaration_span(context, key);
    for module in &context.project.modules {
        let Some(root) = context.project.syntax(module.file) else {
            continue;
        };
        for token in root
            .descendants_with_tokens()
            .filter_map(|item| item.into_token())
            .filter(|token| matches!(token.kind(), SyntaxKind::Ident | SyntaxKind::ThemeName))
        {
            let matches = match key {
                Key::Definition(id) => module.scope.get(token.text()) == Some(id),
                Key::ThemeToken(name) => token.text() == name,
                Key::Member {
                    component, name, ..
                } => {
                    token.text() == name
                        && module
                            .definitions
                            .iter()
                            .find(|definition| definition.id == *component)
                            .is_some_and(|definition| {
                                definition.span.range.contains_range(token.text_range())
                                    && member_declaration
                                        .as_ref()
                                        .is_some_and(|span| is_member_reference(&token, span))
                            })
                }
            };
            if matches {
                output.push(Location {
                    path: database.file_path(module.file).unwrap_or_default().into(),
                    start: token.text_range().start().into(),
                    end: token.text_range().end().into(),
                });
            }
        }
    }
    output.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start.cmp(&right.start))
    });
    output.dedup();
    output
}

/// Maps a definition payload to its editor-facing symbol kind.
fn definition_kind(kind: &DefinitionKind) -> SymbolKind {
    match kind {
        DefinitionKind::Component(_) => SymbolKind::Component,
        DefinitionKind::Struct(_) => SymbolKind::Struct,
        DefinitionKind::Enum(_) => SymbolKind::Enum,
        DefinitionKind::Theme(_) => SymbolKind::Theme,
        DefinitionKind::Style(_) => SymbolKind::Style,
        DefinitionKind::Effect(_) => SymbolKind::Effect,
    }
}

/// Returns the stable human spelling for a symbol kind.
fn kind_label(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Component => "component",
        SymbolKind::Property => "property",
        SymbolKind::Callback => "callback",
        SymbolKind::Slot => "slot",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Theme => "theme",
        SymbolKind::Token => "token",
        SymbolKind::Style => "style",
        SymbolKind::Effect => "effect",
        SymbolKind::Native => "native",
        SymbolKind::Keyword => "keyword",
    }
}

/// Returns a readable data-flow name for a component property.
fn direction_label(direction: PropertyDirection) -> &'static str {
    match direction {
        PropertyDirection::Private => "private",
        PropertyDirection::Input => "input",
        PropertyDirection::Output => "output",
        PropertyDirection::InputOutput => "two-way",
    }
}

/// Validates the grammar's Unicode and kebab-case identifier rules.
fn valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character == '_' || character.is_alphabetic())
        && characters
            .all(|character| character == '_' || character == '-' || character.is_alphanumeric())
}
