# ADR 0001: Engine-owned dynamic names

## Decision

`argui-core` owns `Name`, an immutable value with explicit static and owned
constructors. Static text is borrowed directly and owned text is held by
`Arc<str>`. DSL symbol IDs remain compiler-local; engine APIs use
`Name` for runtime equality, hashing, diagnostics, and lookup.

Static constructors may be `const` only where a distinct static wrapper is
retained. No implementation may leak allocations or require a global mutable
interner. The live runtime may intern `Name` values locally as an optimization.

## Dependency and ownership

Core, UI, paint, and render may depend on `argui-core::Name`. No engine crate
depends on a DSL crate. Public wrappers such as `ActionId`, `EffectId`,
`StateName`, `StateScopeId`, and `ContainerScopeId` own a `Name`.

## Compatibility

Literal call sites use `from_static`; dynamically compiled source uses
`from_owned` or `From<String>`. Equality and hashing are content-based across
both origins.
