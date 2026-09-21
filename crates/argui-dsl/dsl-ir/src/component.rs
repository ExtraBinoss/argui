use std::collections::HashMap;

use argui_dsl_semantic::{DefinitionKind, SemanticProject};
use argui_dsl_syntax::{Span, SyntaxKind};

use crate::{
    AssetId, ComponentId, IrCallback, IrComponent, IrProperty, LowerError, SourceInfo, declaration,
    expression,
    lower::{Tables, no_locals},
    visual::VisualLowerer,
};

/// Lowers every component API and visual tree.
pub(crate) fn components(
    project: &SemanticProject,
    schema: &argui_schema::SchemaRegistry,
    tables: &Tables,
    assets: &mut HashMap<String, AssetId>,
    errors: &mut Vec<LowerError>,
) -> Vec<IrComponent> {
    let mut output = Vec::new();
    for module in &project.modules {
        for definition in &module.definitions {
            let DefinitionKind::Component(value) = &definition.kind else {
                continue;
            };
            let Some(syntax) = declaration::declaration_syntax(project, module.file, definition)
            else {
                errors.push(LowerError::new(
                    "component syntax is unavailable",
                    definition.span,
                ));
                continue;
            };
            let id = tables.components[&definition.id];
            let members = &tables.component_members[&definition.id];
            let locals = no_locals();
            let properties = value
                .properties
                .iter()
                .zip(
                    syntax
                        .children()
                        .filter(|node| node.kind() == SyntaxKind::PropertyDecl),
                )
                .map(|(property, node)| {
                    let mut context = expression::Context {
                        file: module.file,
                        module_path: &module.path,
                        component: Some(id),
                        site: None,
                        properties: &members.properties,
                        callbacks: &members.callbacks,
                        locals: &locals,
                        tokens: &tables.tokens,
                        fields: &tables.named_fields,
                        assets,
                        errors,
                    };
                    IrProperty {
                        id: members.properties[&property.name].0,
                        value_type: crate::IrType::resolved(
                            &property.value_type,
                            &tables.struct_fields,
                        ),
                        direction: property.direction,
                        required: property.required,
                        default: declaration::child_expression(&node)
                            .map(|value| expression::lower(&value, &mut context)),
                        dependencies: property
                            .dependencies
                            .iter()
                            .filter_map(|name| members.properties.get(name).map(|entry| entry.0))
                            .collect(),
                        source: component_source(module.file, id, &node),
                    }
                })
                .collect();
            let callbacks = value
                .callbacks
                .iter()
                .map(|callback| {
                    let member = &members.callbacks[&callback.name];
                    IrCallback {
                        id: member.0,
                        parameters: member.1.clone(),
                        result: member.2.clone(),
                        source: SourceInfo::new(callback.span, Some(id), None),
                    }
                })
                .collect();
            let slots = value
                .slots
                .iter()
                .filter_map(|slot| members.slots.get(&slot.name).copied())
                .collect();
            let template_slots = value
                .slots
                .iter()
                .filter(|slot| slot.template)
                .filter_map(|slot| members.slots.get(&slot.name).copied())
                .collect();
            let mut visual =
                VisualLowerer::new(module, id, members, tables, schema, assets, errors);
            let body = visual.component_body(&syntax);
            let (states, animations) = visual.finish();
            output.push(IrComponent {
                id,
                properties,
                callbacks,
                slots,
                template_slots,
                body,
                states,
                animations,
                source: SourceInfo::new(definition.span, Some(id), None),
            });
        }
    }
    output
}

/// Creates component-scoped source metadata for one syntax node.
fn component_source(
    file: argui_dsl_syntax::FileId,
    component: ComponentId,
    node: &argui_dsl_syntax::SyntaxNode,
) -> SourceInfo {
    SourceInfo::new(Span::new(file, node.text_range()), Some(component), None)
}
