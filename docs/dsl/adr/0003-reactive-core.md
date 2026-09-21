# ADR 0003: Language-independent reactive core

## Decision

`argui-reactive` provides typed `Property<T>`, dependency capture, derived
bindings, observers, transactions, revisions, deterministic cycle detection,
and guarded two-way links. It contains no syntax, IR, module, or dynamic DSL
value types.

Reads during binding evaluation register dependencies. Writes mark dependents
dirty and transactions coalesce notification until the outer commit. Clean
bindings do no work. Re-entrant evaluation reports the complete dependency
cycle rather than recursing.

Generated typed properties and live erased properties adapt to this same
graph, which makes update ordering and batching part of shared semantics.
