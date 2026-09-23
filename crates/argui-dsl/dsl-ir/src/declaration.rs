use std::collections::HashMap;

use argui_dsl_semantic::{Definition, DefinitionKind, Module, SemanticProject};
use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    AssetId, AssetKind, EffectId, IrEffect, IrEffectParameter, IrElementTarget, IrEnum,
    IrEnumVariant, IrExpression, IrModule, IrPropertyBinding, IrStruct, IrStructField, IrStyle,
    IrStyleState, IrTheme, IrThemeMode, IrThemeToken, LowerError, ModuleId, PropertyId,
    PropertyTargetId, SourceInfo, StyleId, StyleStateId, ThemeId, ThemeModeId, VariantId,
    expression, id::derive, id::hash_text, lower::Tables, lower::no_locals,
};

/// Lowers canonical module records in stable source order.
pub(crate) fn modules(project: &SemanticProject) -> Vec<IrModule> {
    let owners = project
        .modules
        .iter()
        .flat_map(|module| {
            module
                .definitions
                .iter()
                .map(move |definition| (definition.id, ModuleId::from_raw(hash_text(&module.path))))
        })
        .collect::<HashMap<_, _>>();
    project
        .modules
        .iter()
        .map(|module| IrModule {
            id: ModuleId::from_raw(hash_text(&module.path)),
            path: module.path.clone(),
            imports: module
                .imports
                .iter()
                .map(|import| {
                    let definitions = import
                        .items
                        .iter()
                        .filter_map(|item| module.scope.get(&item.alias).copied())
                        .collect::<Vec<_>>();
                    let natives = import
                        .items
                        .iter()
                        .filter_map(|item| module.native_scope.get(&item.alias).copied())
                        .collect::<Vec<_>>();
                    crate::IrImport {
                        module: definitions
                            .first()
                            .and_then(|symbol| owners.get(symbol))
                            .copied(),
                        definitions,
                        natives,
                        source: import.span,
                    }
                })
                .collect(),
            definitions: module
                .definitions
                .iter()
                .map(|definition| definition.id)
                .collect(),
        })
        .collect()
}

/// Lowers user structs and their stable fields.
pub(crate) fn structs(project: &SemanticProject, tables: &Tables) -> Vec<IrStruct> {
    definitions(project)
        .filter_map(|(_, definition)| {
            let DefinitionKind::Struct(value) = &definition.kind else {
                return None;
            };
            Some(IrStruct {
                symbol: definition.id,
                fields: value
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| IrStructField {
                        id: tables.struct_fields[&definition.id][index],
                        value_type: crate::IrType::resolved(
                            &field.value_type,
                            &tables.struct_fields,
                        ),
                        source: source(field.span),
                    })
                    .collect(),
                source: source(definition.span),
            })
        })
        .collect()
}

/// Lowers user enums and stable variants.
pub(crate) fn enums(project: &SemanticProject) -> Vec<IrEnum> {
    definitions(project)
        .filter_map(|(_, definition)| {
            let DefinitionKind::Enum(value) = &definition.kind else {
                return None;
            };
            Some(IrEnum {
                symbol: definition.id,
                variants: value
                    .variants
                    .iter()
                    .map(|(name, span)| IrEnumVariant {
                        id: VariantId::from_raw(derive(
                            definition.id.raw(),
                            "variant",
                            hash_text(name),
                        )),
                        source: source(*span),
                    })
                    .collect(),
                source: source(definition.span),
            })
        })
        .collect()
}

/// Lowers theme token defaults, dependencies, and mode overrides.
pub(crate) fn themes(
    project: &SemanticProject,
    tables: &Tables,
    assets: &mut HashMap<String, crate::IrAsset>,
    errors: &mut Vec<LowerError>,
) -> Vec<IrTheme> {
    definitions(project)
        .filter_map(|(module, definition)| {
            let DefinitionKind::Theme(value) = &definition.kind else {
                return None;
            };
            let syntax = declaration_syntax(project, module.file, definition)?;
            let properties = HashMap::new();
            let callbacks = HashMap::new();
            let locals = no_locals();
            let mut context = expression::Context {
                file: module.file,
                module_path: &module.path,
                component: None,
                site: None,
                properties: &properties,
                callbacks: &callbacks,
                locals: &locals,
                tokens: &tables.tokens,
                fields: &tables.named_fields,
                references: None,
                assets,
                errors,
            };
            let nodes = syntax
                .children()
                .filter(|node| node.kind() == SyntaxKind::ThemeTokenDecl)
                .collect::<Vec<_>>();
            let tokens = value
                .tokens
                .iter()
                .zip(&nodes)
                .map(|(token, node)| {
                    let default = if let Some(value) = child_expression(node) {
                        expression::lower(&value, &mut context)
                    } else {
                        missing_expression(node, &mut context)
                    };
                    IrThemeToken {
                        id: tables.tokens[&token.name].0,
                        value_type: crate::IrType::resolved(
                            &token.value_type,
                            &tables.struct_fields,
                        ),
                        dependencies: token
                            .dependencies
                            .iter()
                            .filter_map(|dependency| {
                                tables.tokens.get(dependency).map(|entry| entry.0)
                            })
                            .collect(),
                        default,
                        source: source(token.span),
                    }
                })
                .collect();
            let modes = syntax
                .children()
                .filter(|node| node.kind() == SyntaxKind::ThemeModeDecl)
                .filter_map(|node| {
                    let name = direct_name_or_theme(&node)?;
                    Some(IrThemeMode {
                        id: ThemeModeId::from_raw(hash_text(&name)),
                        overrides: assignments(&node)
                            .filter_map(|assignment| {
                                let name = direct_name_or_theme(&assignment)?;
                                let token = tables.tokens.get(&name)?.0;
                                let value = child_expression(&assignment)
                                    .map(|value| expression::lower(&value, &mut context))?;
                                Some((token, value))
                            })
                            .collect(),
                        source: node_source(module.file, &node),
                    })
                })
                .collect();
            Some(IrTheme {
                id: ThemeId::from_raw(definition.id.raw()),
                tokens,
                modes,
                source: source(definition.span),
            })
        })
        .collect()
}

/// Lowers named styles to resolved native or component property IDs.
pub(crate) fn styles(
    project: &SemanticProject,
    schema: &argui_schema::SchemaRegistry,
    tables: &Tables,
    assets: &mut HashMap<String, crate::IrAsset>,
    errors: &mut Vec<LowerError>,
) -> Vec<IrStyle> {
    definitions(project)
        .filter_map(|(module, definition)| {
            let DefinitionKind::Style(value) = &definition.kind else {
                return None;
            };
            let syntax = declaration_syntax(project, module.file, definition)?;
            let target = resolve_target(module, &value.target, tables)?;
            let (properties, property_targets) = style_properties(module, target, schema, tables);
            let callbacks = HashMap::new();
            let locals = no_locals();
            let mut context = expression::Context {
                file: module.file,
                module_path: &module.path,
                component: None,
                site: None,
                properties: &properties,
                callbacks: &callbacks,
                locals: &locals,
                tokens: &tables.tokens,
                fields: &tables.named_fields,
                references: None,
                assets,
                errors,
            };
            let block = syntax
                .children()
                .find(|node| node.kind() == SyntaxKind::Block)?;
            let properties = direct_assignments(&block)
                .filter_map(|node| lower_style_assignment(&node, &property_targets, &mut context))
                .collect();
            let states = block
                .children()
                .filter(|node| node.kind() == SyntaxKind::StyleStateDecl)
                .filter_map(|node| {
                    let name = direct_name_or_theme(&node)?;
                    Some(IrStyleState {
                        observation: match name.as_str() {
                            "hover" => crate::IrObservation::Hover,
                            "pressed" => crate::IrObservation::Pressed,
                            "focus" => crate::IrObservation::Focused,
                            _ => crate::IrObservation::FocusVisible,
                        },
                        id: StyleStateId::from_raw(derive(
                            definition.id.raw(),
                            "style-state",
                            hash_text(&name),
                        )),
                        properties: assignments(&node)
                            .filter_map(|assignment| {
                                lower_style_assignment(&assignment, &property_targets, &mut context)
                            })
                            .collect(),
                        source: node_source(module.file, &node),
                    })
                })
                .collect();
            Some(IrStyle {
                id: StyleId::from_raw(definition.id.raw()),
                target,
                properties,
                states,
                source: source(definition.span),
            })
        })
        .collect()
}

/// Lowers effect shaders and typed parameter defaults.
pub(crate) fn effects(
    project: &SemanticProject,
    tables: &Tables,
    assets: &mut HashMap<String, crate::IrAsset>,
    errors: &mut Vec<LowerError>,
) -> Vec<IrEffect> {
    definitions(project)
        .filter_map(|(module, definition)| {
            let DefinitionKind::Effect(value) = &definition.kind else {
                return None;
            };
            let syntax = declaration_syntax(project, module.file, definition)?;
            let path = resolve_asset_path(&module.path, value.shader.as_ref()?);
            let shader = register_asset(assets, &path);
            let properties = HashMap::new();
            let callbacks = HashMap::new();
            let locals = no_locals();
            let mut context = expression::Context {
                file: module.file,
                module_path: &module.path,
                component: None,
                site: None,
                properties: &properties,
                callbacks: &callbacks,
                locals: &locals,
                tokens: &tables.tokens,
                fields: &tables.named_fields,
                references: None,
                assets,
                errors,
            };
            let nodes = syntax
                .children()
                .filter(|node| node.kind() == SyntaxKind::EffectParameterDecl)
                .collect::<Vec<_>>();
            let parameters = value
                .parameters
                .iter()
                .zip(nodes)
                .map(|(parameter, node)| IrEffectParameter {
                    id: PropertyId::from_raw(derive(
                        definition.id.raw(),
                        "effect-parameter",
                        hash_text(&parameter.name),
                    )),
                    name: parameter.name.clone(),
                    value_type: crate::IrType::resolved(
                        &parameter.value_type,
                        &tables.struct_fields,
                    ),
                    default: child_expression(&node)
                        .map(|value| expression::lower(&value, &mut context)),
                    source: source(parameter.span),
                })
                .collect();
            Some(IrEffect {
                id: EffectId::from_raw(definition.id.raw()),
                shader,
                bounded_damage: value.bounded_damage,
                parameters,
                source: source(definition.span),
            })
        })
        .collect()
}

/// Classifies one collected asset by its normalized extension.
pub(crate) fn asset(id: AssetId, path: String) -> crate::IrAsset {
    let lower = path.to_ascii_lowercase();
    let kind = if lower.ends_with(".wgsl") {
        AssetKind::Shader
    } else if lower.ends_with(".svg") {
        AssetKind::Vector
    } else if [".png", ".jpg", ".jpeg", ".webp", ".gif"]
        .iter()
        .any(|extension| lower.ends_with(extension))
    {
        AssetKind::Image
    } else {
        AssetKind::Other
    };
    crate::IrAsset {
        id,
        path,
        kind,
        inline_bytes: None,
    }
}

/// Resolves one style target through the module's native and symbol scopes.
fn resolve_target(module: &Module, name: &str, tables: &Tables) -> Option<IrElementTarget> {
    module
        .native_scope
        .get(name)
        .copied()
        .map(IrElementTarget::Native)
        .or_else(|| {
            module
                .scope
                .get(name)
                .and_then(|symbol| tables.components.get(symbol))
                .copied()
                .map(IrElementTarget::Component)
        })
}

/// Builds the readable and target-ID maps for a style's target type.
fn style_properties(
    module: &Module,
    target: IrElementTarget,
    schema: &argui_schema::SchemaRegistry,
    tables: &Tables,
) -> (
    HashMap<String, (PropertyId, crate::IrType)>,
    HashMap<String, PropertyTargetId>,
) {
    match target {
        IrElementTarget::Component(component) => {
            let symbol = module.scope.values().find(|symbol| {
                tables
                    .components
                    .get(symbol)
                    .is_some_and(|value| *value == component)
            });
            let properties = symbol
                .and_then(|symbol| tables.component_members.get(symbol))
                .map(|members| members.properties.clone())
                .unwrap_or_default();
            let targets = properties
                .iter()
                .map(|(name, (id, _))| (name.clone(), PropertyTargetId::Component(*id)))
                .collect();
            (properties, targets)
        }
        IrElementTarget::Native(native) => {
            let Some(native) = schema.schema(native) else {
                return (HashMap::new(), HashMap::new());
            };
            let targets = native
                .properties
                .iter()
                .map(|property| {
                    (
                        property.name.as_str().to_string(),
                        PropertyTargetId::Native(property.id),
                    )
                })
                .collect();
            (HashMap::new(), targets)
        }
    }
}

/// Lowers one style assignment after schema/member resolution.
fn lower_style_assignment(
    node: &SyntaxNode,
    targets: &HashMap<String, PropertyTargetId>,
    context: &mut expression::Context<'_>,
) -> Option<IrPropertyBinding> {
    let name = direct_name_or_theme(node)?;
    let target = *targets.get(&name)?;
    let value = child_expression(node).map(|node| expression::lower(&node, context))?;
    Some(IrPropertyBinding {
        target,
        value,
        two_way: node.kind() == SyntaxKind::TwoWayBinding,
        source: node_source(context.file, node),
    })
}

/// Registers an asset path and returns its content-addressed identity.
pub(crate) fn register_asset(assets: &mut HashMap<String, crate::IrAsset>, path: &str) -> AssetId {
    assets
        .entry(path.to_string())
        .or_insert_with(|| asset(AssetId::from_raw(hash_text(path)), path.to_string()))
        .id
}

/// Resolves an asset path lexically relative to its declaring module.
pub(crate) fn resolve_asset_path(module_path: &str, asset_path: &str) -> String {
    if asset_path.starts_with('@') || asset_path.starts_with('/') {
        return asset_path.to_string();
    }
    let base = module_path.rsplit_once('/').map_or("", |(base, _)| base);
    let joined = if base.is_empty() {
        asset_path.to_string()
    } else {
        format!("{base}/{asset_path}")
    };
    let mut components = Vec::new();
    for component in joined.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            value => components.push(value),
        }
    }
    components.join("/")
}

/// Finds the lossless declaration node corresponding to a semantic definition.
pub(crate) fn declaration_syntax(
    project: &SemanticProject,
    file: FileId,
    definition: &Definition,
) -> Option<SyntaxNode> {
    project
        .syntax(file)?
        .children()
        .find(|node| node.text_range() == definition.span.range)
}

/// Iterates all definitions with their owning module.
fn definitions(project: &SemanticProject) -> impl Iterator<Item = (&Module, &Definition)> {
    project.modules.iter().flat_map(|module| {
        module
            .definitions
            .iter()
            .map(move |definition| (module, definition))
    })
}

/// Iterates every descendant assignment.
fn assignments(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> + '_ {
    node.descendants().filter(|node| {
        matches!(
            node.kind(),
            SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
        )
    })
}

/// Iterates assignments directly inside a property block.
fn direct_assignments(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> + '_ {
    node.children().filter(|node| {
        matches!(
            node.kind(),
            SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
        )
    })
}

/// Returns a node's direct expression child.
pub(crate) fn child_expression(node: &SyntaxNode) -> Option<SyntaxNode> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::Expr)
}

/// Returns a direct identifier or theme-token spelling.
pub(crate) fn direct_name_or_theme(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| {
            matches!(
                token.kind(),
                SyntaxKind::Ident | SyntaxKind::ThemeName | SyntaxKind::FromKw
            )
        })
        .map(|token| token.text().to_string())
}

/// Creates source metadata without a component or visual site.
fn source(span: Span) -> SourceInfo {
    SourceInfo::new(span, None, None)
}

/// Creates source metadata for a syntax node outside a component.
fn node_source(file: FileId, node: &SyntaxNode) -> SourceInfo {
    source(Span::new(file, node.text_range()))
}

/// Produces an error expression for a missing initializer.
fn missing_expression(node: &SyntaxNode, context: &mut expression::Context<'_>) -> IrExpression {
    context.errors.push(LowerError::new(
        "declaration initializer is missing",
        Span::new(context.file, node.text_range()),
    ));
    let source = node_source(context.file, node);
    IrExpression {
        id: expression::expression_id(&source),
        value_type: crate::IrType::Unknown,
        kind: crate::IrExpressionKind::Constant(crate::IrValue::Null),
        source,
    }
}
