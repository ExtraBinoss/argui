use crate::{Diagnostic, DiagnosticCode, lower::LoweredModule};

/// Detects deterministic import cycles and reports each cycle once.
pub(super) fn import_cycles(
    modules: &[LoweredModule],
    edges: &[Vec<usize>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut states = vec![Visit::Fresh; modules.len()];
    let mut stack = Vec::new();
    for module in 0..modules.len() {
        visit(module, modules, edges, &mut states, &mut stack, diagnostics);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Visit {
    Fresh,
    Active,
    Done,
}

/// Depth-first visits one module and emits a source-ordered cycle path.
fn visit(
    module: usize,
    modules: &[LoweredModule],
    edges: &[Vec<usize>],
    states: &mut [Visit],
    stack: &mut Vec<usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match states[module] {
        Visit::Done => return,
        Visit::Active => {
            let start = stack
                .iter()
                .position(|candidate| *candidate == module)
                .unwrap_or(0);
            let cycle = stack[start..]
                .iter()
                .chain([&module])
                .map(|index| modules[*index].path.as_str())
                .collect::<Vec<_>>()
                .join(" -> ");
            let span = modules[module].imports.first().map_or_else(
                || {
                    argui_dsl_syntax::Span::new(
                        modules[module].file,
                        argui_dsl_syntax::TextRange::empty(0.into()),
                    )
                },
                |import| import.span,
            );
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ImportCycle,
                format!("import cycle: {cycle}"),
                span,
            ));
            return;
        }
        Visit::Fresh => {}
    }
    states[module] = Visit::Active;
    stack.push(module);
    for target in &edges[module] {
        visit(*target, modules, edges, states, stack, diagnostics);
    }
    stack.pop();
    states[module] = Visit::Done;
}
