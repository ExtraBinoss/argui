use std::collections::{HashMap, HashSet};

use argui_dsl_syntax::Span;

use crate::{Diagnostic, DiagnosticCode};

pub(super) fn report_named_cycles(
    graph: &HashMap<String, Vec<String>>,
    spans: &HashMap<String, Span>,
    code: DiagnosticCode,
    label: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = graph.keys().cloned().collect::<Vec<_>>();
    names.sort();
    let mut done = HashSet::new();
    for name in names {
        let mut stack = Vec::new();
        visit_named(
            &name,
            graph,
            spans,
            code,
            label,
            &mut stack,
            &mut done,
            diagnostics,
        );
    }
}

/// Depth-first visits one named dependency and reports an active-stack cycle.
#[allow(clippy::too_many_arguments)]
fn visit_named(
    name: &str,
    graph: &HashMap<String, Vec<String>>,
    spans: &HashMap<String, Span>,
    code: DiagnosticCode,
    label: &str,
    stack: &mut Vec<String>,
    done: &mut HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if done.contains(name) {
        return;
    }
    if let Some(start) = stack.iter().position(|value| value == name) {
        let mut cycle = stack[start..].to_vec();
        cycle.push(name.into());
        if let Some(span) = spans.get(name).copied() {
            diagnostics.push(Diagnostic::error(
                code,
                format!("{label}: {}", cycle.join(" -> ")),
                span,
            ));
        }
        return;
    }
    stack.push(name.into());
    if let Some(dependencies) = graph.get(name) {
        for dependency in dependencies {
            if graph.contains_key(dependency) {
                visit_named(
                    dependency,
                    graph,
                    spans,
                    code,
                    label,
                    stack,
                    done,
                    diagnostics,
                );
            }
        }
    }
    stack.pop();
    done.insert(name.into());
}
