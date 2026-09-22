//! Native output references resolved from explicit visual source identities.

use argui_dsl_semantic::Module;
use argui_dsl_syntax::{SyntaxKind, SyntaxNode};

use crate::{
    ComponentId, IrType, SiteId,
    expression::{self, References},
    lower::Tables,
    site,
};

use super::direct_identifier;

/// Indexes unique, explicitly identified native sites for host effects.
///
/// `component` owns the retained identities, `module` resolves native names,
/// and `node` is its visual syntax. Returns source IDs and stable site IDs.
pub(super) fn collect_sites(
    component: ComponentId,
    module: &Module,
    node: &SyntaxNode,
) -> std::collections::HashMap<String, SiteId> {
    node.descendants()
        .filter(|child| child.kind() == SyntaxKind::Element)
        .filter(|element| {
            !element
                .ancestors()
                .any(|ancestor| ancestor.kind() == SyntaxKind::ForExpr)
        })
        .filter_map(|element| {
            let name = site::explicit_id(&element)?;
            let native = direct_identifier(&element)?;
            module
                .native_scope
                .contains_key(&native)
                .then(|| (name, site::identify(component, &element)))
        })
        .collect()
}

/// Resolves native observations and child outputs on identified elements.
///
/// * `component` — stable owner used to derive each source site's identity.
/// * `module` — scope containing imported native element names.
/// * `schema` — registry defining observable properties and their types.
/// * `tables` — stable IDs and types of imported child component members.
/// * `node` — syntax of the component being lowered.
///
/// Returns source-name-to-property lookup data for typed expression lowering.
/// Repeated elements are excluded because one name cannot identify many instances.
pub(crate) fn collect(
    component: ComponentId,
    module: &Module,
    schema: &argui_schema::SchemaRegistry,
    tables: &Tables,
    node: &SyntaxNode,
) -> References {
    let mut references = References::new();
    for element in node
        .descendants()
        .filter(|child| child.kind() == SyntaxKind::Element)
    {
        let Some(name) = site::explicit_id(&element) else {
            continue;
        };
        if element
            .ancestors()
            .any(|ancestor| ancestor.kind() == SyntaxKind::ForExpr)
        {
            continue;
        }
        let Some(element_name) = direct_identifier(&element) else {
            continue;
        };
        let site = site::identify(component, &element);
        let properties = if let Some(native) = module
            .native_scope
            .get(&element_name)
            .and_then(|id| schema.schema(*id))
        {
            native
                .properties
                .iter()
                .filter_map(|property| {
                    let observation = property.observation?;
                    let value_type = match observation {
                        argui_schema::ObservationKind::Hover
                        | argui_schema::ObservationKind::Pressed
                        | argui_schema::ObservationKind::Focused
                        | argui_schema::ObservationKind::FocusVisible => IrType::Bool,
                        _ => IrType::Length,
                    };
                    Some((
                        property.name.as_str().to_string(),
                        expression::ReferenceProperty::Observed {
                            site,
                            property: property.id,
                            observation: observation.into(),
                            value_type,
                        },
                    ))
                })
                .collect()
        } else if let Some(members) = module
            .scope
            .get(&element_name)
            .and_then(|id| tables.component_members.get(id))
        {
            if element
                .ancestors()
                .any(|ancestor| ancestor.kind() == SyntaxKind::IfExpr)
            {
                continue;
            }
            members
                .properties
                .iter()
                .map(|(name, (property, value_type))| {
                    (
                        name.clone(),
                        expression::ReferenceProperty::Child {
                            site,
                            property: *property,
                            value_type: value_type.clone(),
                        },
                    )
                })
                .collect()
        } else {
            continue;
        };
        references.insert(name, properties);
    }
    references
}
