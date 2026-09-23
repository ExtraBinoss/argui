use std::collections::HashMap;

use argui_dsl_semantic::{DefinitionKind, SemanticProject, SymbolId};

use crate::{
    CallbackId, ComponentId, FieldId, IrProject, IrType, LocalId, LowerError, PropertyId, SlotId,
    TokenId, component, declaration, id::derive, id::hash_text,
};

/// Resolved public members of one DSL component.
pub(crate) struct ComponentMembers {
    pub properties: HashMap<String, (PropertyId, IrType)>,
    pub callbacks: HashMap<String, (CallbackId, Vec<IrType>, IrType)>,
    pub slots: HashMap<String, SlotId>,
}

/// Immutable stable-ID indices shared by all lowering passes.
pub(crate) struct Tables {
    pub components: HashMap<SymbolId, ComponentId>,
    pub component_members: HashMap<SymbolId, ComponentMembers>,
    pub struct_fields: HashMap<SymbolId, Vec<FieldId>>,
    pub named_fields: HashMap<SymbolId, HashMap<String, (FieldId, IrType)>>,
    pub tokens: HashMap<String, (TokenId, IrType)>,
}

/// Lowers a valid semantic project to the stable, name-free backend IR.
///
/// * `project` — resolved semantic snapshot retaining its lossless syntax trees.
/// * `schema` — the exact native schema registry used for semantic checking.
///
/// # Errors
///
/// Returns source-located invariant failures when the semantic project is invalid,
/// a schema differs from the checked schema, or a syntax site cannot be normalized.
pub fn lower(
    project: &SemanticProject,
    schema: &argui_schema::SchemaRegistry,
) -> Result<IrProject, Vec<LowerError>> {
    if !project.is_valid() {
        return Err(project
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == argui_dsl_semantic::Severity::Error)
            .map(|diagnostic| LowerError::new(diagnostic.message.clone(), diagnostic.primary))
            .collect());
    }
    let tables = Tables::build(project);
    let mut errors = Vec::new();
    let mut assets = HashMap::<String, crate::IrAsset>::new();
    let modules = declaration::modules(project);
    let structs = declaration::structs(project, &tables);
    let enums = declaration::enums(project);
    let themes = declaration::themes(project, &tables, &mut assets, &mut errors);
    let styles = declaration::styles(project, schema, &tables, &mut assets, &mut errors);
    let effects = declaration::effects(project, &tables, &mut assets, &mut errors);
    let components = component::components(
        project,
        schema,
        &tables,
        &effects,
        &styles,
        &mut assets,
        &mut errors,
    );
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut assets = assets.into_values().collect::<Vec<_>>();
    assets.sort_unstable_by_key(|asset| asset.id);
    Ok(IrProject {
        modules,
        structs,
        enums,
        components,
        themes,
        styles,
        effects,
        assets,
    })
}

impl Tables {
    /// Builds stable IDs for every declaration member before expressions are lowered.
    fn build(project: &SemanticProject) -> Self {
        let mut components = HashMap::new();
        let mut struct_fields: HashMap<SymbolId, Vec<FieldId>> = HashMap::new();
        for definition in project
            .modules
            .iter()
            .flat_map(|module| &module.definitions)
        {
            match &definition.kind {
                DefinitionKind::Component(_) => {
                    components.insert(definition.id, ComponentId::from_raw(definition.id.raw()));
                }
                DefinitionKind::Struct(value) => {
                    struct_fields.insert(
                        definition.id,
                        value
                            .fields
                            .iter()
                            .map(|field| {
                                FieldId::from_raw(derive(
                                    definition.id.raw(),
                                    "field",
                                    hash_text(&field.name),
                                ))
                            })
                            .collect(),
                    );
                }
                _ => {}
            }
        }
        let mut named_fields = HashMap::new();
        let mut component_members = HashMap::new();
        let mut tokens = HashMap::new();
        for definition in project
            .modules
            .iter()
            .flat_map(|module| &module.definitions)
        {
            match &definition.kind {
                DefinitionKind::Struct(value) => {
                    let fields = value
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let id = struct_fields[&definition.id][index];
                            (
                                field.name.clone(),
                                (id, IrType::resolved(&field.value_type, &struct_fields)),
                            )
                        })
                        .collect();
                    named_fields.insert(definition.id, fields);
                }
                DefinitionKind::Component(value) => {
                    component_members.insert(
                        definition.id,
                        component_members_for(definition.id, value, &struct_fields),
                    );
                }
                DefinitionKind::Theme(value) => {
                    for token in &value.tokens {
                        tokens.insert(
                            token.name.clone(),
                            (
                                TokenId::from_raw(derive(
                                    definition.id.raw(),
                                    "token",
                                    hash_text(&token.name),
                                )),
                                IrType::resolved(&token.value_type, &struct_fields),
                            ),
                        );
                    }
                }
                _ => {}
            }
        }
        Self {
            components,
            component_members,
            struct_fields,
            named_fields,
            tokens,
        }
    }
}

/// Derives all stable component member IDs and their resolved types.
fn component_members_for(
    symbol: SymbolId,
    value: &argui_dsl_semantic::ComponentDefinition,
    fields: &HashMap<SymbolId, Vec<FieldId>>,
) -> ComponentMembers {
    let properties = value
        .properties
        .iter()
        .map(|property| {
            (
                property.name.clone(),
                (
                    PropertyId::from_raw(derive(
                        symbol.raw(),
                        "property",
                        hash_text(&property.name),
                    )),
                    IrType::resolved(&property.value_type, fields),
                ),
            )
        })
        .collect();
    let callbacks = value
        .callbacks
        .iter()
        .map(|callback| {
            (
                callback.name.clone(),
                (
                    CallbackId::from_raw(derive(
                        symbol.raw(),
                        "callback",
                        hash_text(&callback.name),
                    )),
                    callback
                        .parameters
                        .iter()
                        .map(|parameter| IrType::resolved(&parameter.value_type, fields))
                        .collect(),
                    IrType::resolved(&callback.result, fields),
                ),
            )
        })
        .collect();
    let slots = value
        .slots
        .iter()
        .map(|slot| {
            (
                slot.name.clone(),
                SlotId::from_raw(derive(symbol.raw(), "slot", hash_text(&slot.name))),
            )
        })
        .collect();
    ComponentMembers {
        properties,
        callbacks,
        slots,
    }
}

/// Returns an empty local environment for declaration-level expressions.
pub(crate) fn no_locals() -> HashMap<String, (LocalId, IrType)> {
    HashMap::new()
}
