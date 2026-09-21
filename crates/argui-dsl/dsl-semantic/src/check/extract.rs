use std::collections::HashMap;

use argui_dsl_syntax::FileId;

use crate::{
    Definition, DefinitionKind, Diagnostic, DiagnosticCode, SymbolId, Type,
    lower::{LoweredDefinition, LoweredKind, LoweredModule},
};

use super::Scope;
use declaration::{
    component_definition, effect_definition, enum_definition, struct_definition, style_definition,
    theme_definition,
};
use named_cycle::report_named_cycles;
use theme::validate_theme_values;
use visual::validate_component;

mod animation;
mod declaration;
mod named_cycle;
mod theme;
mod visual;

pub(super) fn definition(
    lowered: &LoweredDefinition,
    file: FileId,
    scope: &Scope,
    modules: &[LoweredModule],
    diagnostics: &mut Vec<Diagnostic>,
) -> Definition {
    let kind = match lowered.kind {
        LoweredKind::Struct => DefinitionKind::Struct(struct_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Enum => {
            DefinitionKind::Enum(enum_definition(&lowered.syntax, file, diagnostics))
        }
        LoweredKind::Component => DefinitionKind::Component(component_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Theme => DefinitionKind::Theme(theme_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
        LoweredKind::Style => DefinitionKind::Style(style_definition(&lowered.syntax)),
        LoweredKind::Effect => DefinitionKind::Effect(effect_definition(
            &lowered.syntax,
            file,
            scope,
            modules,
            diagnostics,
        )),
    };
    Definition {
        id: lowered.id,
        name: lowered.name.clone(),
        exported: lowered.exported,
        span: lowered.span,
        kind,
    }
}

/// Builds the cross-theme token type index used by expressions and styles.
pub(super) fn theme_token_index(
    definitions: &HashMap<SymbolId, Definition>,
) -> HashMap<String, Type> {
    definitions
        .values()
        .filter_map(|definition| match &definition.kind {
            DefinitionKind::Theme(theme) => Some(&theme.tokens),
            _ => None,
        })
        .flatten()
        .map(|token| (token.name.clone(), token.value_type.clone()))
        .collect()
}

/// Runs declaration-body and visual-tree validation for one resolved module.
pub(super) fn validate_module(
    module: &LoweredModule,
    scope: &Scope,
    definitions: &HashMap<SymbolId, Definition>,
    theme_tokens: &HashMap<String, Type>,
    schema: &argui_schema::SchemaRegistry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for lowered in &module.definitions {
        let Some(definition) = definitions.get(&lowered.id) else {
            continue;
        };
        match &definition.kind {
            DefinitionKind::Component(component) => validate_component(
                module.file,
                &lowered.syntax,
                component,
                scope,
                definitions,
                theme_tokens,
                schema,
                diagnostics,
            ),
            DefinitionKind::Theme(theme) => validate_theme_values(
                module.file,
                &lowered.syntax,
                theme,
                definitions,
                theme_tokens,
                diagnostics,
            ),
            DefinitionKind::Effect(effect)
                if effect
                    .shader
                    .as_ref()
                    .is_none_or(|path| !path.ends_with(".wgsl")) =>
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidEffect,
                    "effect shader must reference a .wgsl file",
                    lowered.span,
                ));
            }
            _ => {}
        }
    }
}

/// Reports static cycles between component property default bindings.
pub(super) fn binding_cycles(
    definitions: &HashMap<SymbolId, Definition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for definition in definitions.values() {
        let DefinitionKind::Component(component) = &definition.kind else {
            continue;
        };
        let graph = component
            .properties
            .iter()
            .map(|property| (property.name.clone(), property.dependencies.clone()))
            .collect::<HashMap<_, _>>();
        report_named_cycles(
            &graph,
            &component
                .properties
                .iter()
                .map(|property| (property.name.clone(), property.span))
                .collect(),
            DiagnosticCode::BindingCycle,
            "binding cycle",
            diagnostics,
        );
    }
}

/// Reports cycles between derived theme tokens.
pub(super) fn theme_cycles(
    definitions: &HashMap<SymbolId, Definition>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for definition in definitions.values() {
        let DefinitionKind::Theme(theme) = &definition.kind else {
            continue;
        };
        report_named_cycles(
            &theme
                .tokens
                .iter()
                .map(|token| (token.name.clone(), token.dependencies.clone()))
                .collect(),
            &theme
                .tokens
                .iter()
                .map(|token| (token.name.clone(), token.span))
                .collect(),
            DiagnosticCode::ThemeCycle,
            "theme token cycle",
            diagnostics,
        );
    }
}
