mod cycle;
mod expression;
mod extract;

use std::collections::{HashMap, HashSet};

use argui_dsl_syntax::Span;

use crate::{
    Diagnostic, DiagnosticCode, Module, SemanticProject, SymbolId,
    lower::{InputModule, LoweredDefinition, LoweredKind, LoweredModule},
    path,
};

#[derive(Default)]
pub(super) struct Scope {
    pub symbols: HashMap<String, SymbolId>,
    pub natives: HashMap<String, argui_schema::NativeTypeId>,
}

/// Builds and validates one immutable semantic project snapshot.
pub(crate) fn project(
    inputs: Vec<InputModule>,
    schema: &argui_schema::SchemaRegistry,
) -> SemanticProject {
    let mut diagnostics = parse_diagnostics(&inputs);
    let lowered = inputs
        .into_iter()
        .map(crate::lower::module)
        .collect::<Vec<_>>();
    let path_indices = lowered
        .iter()
        .enumerate()
        .map(|(index, module)| (module.path.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut scopes = local_scopes(&lowered, &mut diagnostics);
    let import_edges = resolve_imports(
        &lowered,
        &path_indices,
        schema,
        &mut scopes,
        &mut diagnostics,
    );
    cycle::import_cycles(&lowered, &import_edges, &mut diagnostics);

    let mut definitions = HashMap::new();
    for (index, module) in lowered.iter().enumerate() {
        for definition in &module.definitions {
            let value = extract::definition(
                definition,
                module.file,
                &scopes[index],
                &lowered,
                &mut diagnostics,
            );
            definitions.insert(value.id, value);
        }
    }
    let theme_tokens = extract::theme_token_index(&definitions);
    for (index, module) in lowered.iter().enumerate() {
        extract::validate_module(
            module,
            &scopes[index],
            &definitions,
            &theme_tokens,
            schema,
            &mut diagnostics,
        );
    }
    extract::binding_cycles(&definitions, &mut diagnostics);
    extract::theme_cycles(&definitions, &mut diagnostics);

    let syntax = lowered
        .iter()
        .map(|module| (module.file, module.green.clone()))
        .collect();
    let modules = lowered
        .into_iter()
        .enumerate()
        .map(|(index, lowered)| Module {
            file: lowered.file,
            path: lowered.path,
            imports: lowered.imports,
            definitions: lowered
                .definitions
                .iter()
                .filter_map(|definition| definitions.get(&definition.id).cloned())
                .collect(),
            scope: scopes[index].symbols.clone(),
            native_scope: scopes[index].natives.clone(),
        })
        .collect();
    diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic.primary.file.raw(),
            u32::from(diagnostic.primary.range.start()),
            diagnostic.severity,
        )
    });
    SemanticProject::new(modules, diagnostics, syntax)
}

/// Converts parser diagnostics for every file into the shared diagnostic model.
fn parse_diagnostics(inputs: &[InputModule]) -> Vec<Diagnostic> {
    inputs
        .iter()
        .flat_map(|input| {
            input.parse.diagnostics().iter().map(|diagnostic| {
                Diagnostic::error(
                    DiagnosticCode::Parse,
                    diagnostic.message.clone(),
                    Span::new(input.file, diagnostic.range),
                )
            })
        })
        .collect()
}

/// Creates local symbol scopes and reports duplicate top-level definitions.
fn local_scopes(modules: &[LoweredModule], diagnostics: &mut Vec<Diagnostic>) -> Vec<Scope> {
    modules
        .iter()
        .map(|module| {
            let mut scope = Scope::default();
            let mut spans = HashMap::new();
            for definition in &module.definitions {
                if let Some(previous) = spans.insert(definition.name.clone(), definition.span) {
                    diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::DuplicateDefinition,
                            format!("`{}` is defined more than once", definition.name),
                            definition.span,
                        )
                        .related(previous, "first definition is here"),
                    );
                } else {
                    scope.symbols.insert(definition.name.clone(), definition.id);
                }
            }
            scope
        })
        .collect()
}

/// Resolves imports into per-module scopes and returns local-module graph edges.
fn resolve_imports(
    modules: &[LoweredModule],
    path_indices: &HashMap<String, usize>,
    schema: &argui_schema::SchemaRegistry,
    scopes: &mut [Scope],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Vec<usize>> {
    let mut edges = vec![Vec::new(); modules.len()];
    for (index, module) in modules.iter().enumerate() {
        for import in &module.imports {
            if import.source == "@argui/native" {
                for item in &import.items {
                    if let Some(native) = schema.schema_named(&item.name) {
                        scopes[index].natives.insert(item.alias.clone(), native.id);
                    } else {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::UnresolvedImport,
                            format!("`{}` is not exported by @argui/native", item.name),
                            item.span,
                        ));
                    }
                }
                continue;
            }
            if import.source == "@argui/icons" {
                for item in &import.items {
                    if !cfg!(feature = "icons") {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::UnresolvedImport,
                            "@argui/icons requires the `icons` feature",
                            item.span,
                        ));
                        continue;
                    }
                    let target_path = format!("@argui/icons/{}.argui", item.name);
                    let Some(target) = path_indices.get(&target_path).copied() else {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::UnresolvedImport,
                            format!("`{}` is not exported by @argui/icons", item.name),
                            item.span,
                        ));
                        continue;
                    };
                    let Some(definition) = modules[target]
                        .definitions
                        .iter()
                        .find(|definition| definition.name == item.name && definition.exported)
                    else {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::UnresolvedImport,
                            format!("`{}` is not exported by @argui/icons", item.name),
                            item.span,
                        ));
                        continue;
                    };
                    if target != index {
                        edges[index].push(target);
                    }
                    scopes[index]
                        .symbols
                        .insert(item.alias.clone(), definition.id);
                }
                continue;
            }
            if import.source == "@argui/ui" {
                for item in &import.items {
                    if let Some((target, definition)) =
                        modules.iter().enumerate().find_map(|(target, standard)| {
                            standard
                                .path
                                .starts_with("@argui/ui/")
                                .then(|| {
                                    standard
                                        .definitions
                                        .iter()
                                        .find(|definition| {
                                            definition.name == item.name && definition.exported
                                        })
                                        .map(|definition| (target, definition))
                                })
                                .flatten()
                        })
                    {
                        if target != index {
                            edges[index].push(target);
                        }
                        scopes[index]
                            .symbols
                            .insert(item.alias.clone(), definition.id);
                    } else if let Some(native) = schema.schema_named(&item.name) {
                        scopes[index].natives.insert(item.alias.clone(), native.id);
                    } else {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::UnresolvedImport,
                            format!("`{}` is not exported by @argui/ui", item.name),
                            item.span,
                        ));
                    }
                }
                continue;
            }
            let Some(target_path) = path::resolve(&module.path, &import.source) else {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedImport,
                    format!("invalid import path `{}`", import.source),
                    import.span,
                ));
                continue;
            };
            let Some(target_index) = path_indices.get(&target_path).copied() else {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedImport,
                    format!("module `{target_path}` was not found"),
                    import.span,
                ));
                continue;
            };
            edges[index].push(target_index);
            let target = &modules[target_index];
            for item in &import.items {
                match target
                    .definitions
                    .iter()
                    .find(|definition| definition.name == item.name)
                {
                    Some(definition) if definition.exported => {
                        scopes[index]
                            .symbols
                            .insert(item.alias.clone(), definition.id);
                    }
                    Some(definition) => diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::PrivateImport,
                            format!("`{}` is private to `{target_path}`", item.name),
                            item.span,
                        )
                        .related(definition.span, "private definition is here"),
                    ),
                    None => diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnresolvedImport,
                        format!("`{}` is not exported by `{target_path}`", item.name),
                        item.span,
                    )),
                }
            }
        }
    }
    for edge in &mut edges {
        let mut seen = HashSet::new();
        edge.retain(|target| seen.insert(*target));
    }
    edges
}

/// Returns a lowered definition by stable ID.
pub(super) fn lowered_definition(
    modules: &[LoweredModule],
    id: SymbolId,
) -> Option<&LoweredDefinition> {
    modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.id == id)
}

/// Returns whether a lowered declaration is a user-defined type.
pub(super) fn is_type_kind(kind: LoweredKind) -> bool {
    matches!(kind, LoweredKind::Struct | LoweredKind::Enum)
}
