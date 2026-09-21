# ADR 0005: Declarative native schema

## Decision

`argui-schema` is an engine-facing registry of stable native element,
property, event, slot, variant, and style-part IDs with typed value schemas and
documentation metadata. It does not depend on parser or compiler crates.

Native adapters validate schema values and construct engine elements through
direct adapter functions. The semantic compiler, live adapters, LSP,
documentation generator, and AI schema command consume the same registry.
Compiler code must not dispatch on widget names.

Only behavior-heavy primitives remain native; official visual composition
migrates to the `@argui/ui` DSL package.
