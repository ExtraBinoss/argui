# Argui DSL implementation checklist

This directory contains the language toolchain and its standard library. The
authoritative plan is copied into
[`docs/dsl/ARGUI_DSL_IMPLEMENTATION_PLAN.md`](../../docs/dsl/ARGUI_DSL_IMPLEMENTATION_PLAN.md).
The implementation follows the phases below in order; a phase is checked only
after its exit criteria and targeted tests pass.

## Global invariants

- [x] Live and AOT execution consume the same typed IR semantics.
- [x] Release applications do not link compiler, interpreter, watcher, LSP, or protocol server code.
- [x] Engine crates never depend on `argui-dsl-*` crates.
- [x] UI, WGSL, and asset reloads are transactional and preserve the previous valid generation on failure.
- [x] Every Rust source file stays at or below 600 physical lines; tests mirror `src/` under `tests/`.
- [x] Every new function and method has accurate Rustdoc; no placeholders or deferred branches remain.
- [x] Targeted tests use `--all-features`; the full quality gate is run exactly once immediately before commit.

## Phase checklist

- [x] **A — Architecture contracts.** Ten ADRs define names, retained identity, reactivity, Theme v2, native schema, effects, shader validation, assets, protocol compatibility, and public ABI boundaries.
- [x] **B — Dynamic engine names.** `Name` accepts static and owned strings without leaks; every DSL-facing engine identifier uses it.
- [x] **C — Retained source identity.** Private source-site identity participates in reconciliation and preserves focus, selection, editing, scroll, overlays, and transitions across unrelated insertion/reordering.
- [x] **D — `argui-reactive`.** Typed properties, dependency capture, transactions, deterministic cycle errors, observers, and two-way links are language-independent and idle when clean.
- [x] **E — Theme v2.** Typed token definitions/overrides, derived-token cycle checks, runtime modes, and selective invalidation work from Rust alone.
- [x] **F — Declarative native schema.** Schema IDs and typed values can instantiate native primitives without compiler-side widget-name switches.
- [x] **G — Shader/effect readiness.** Renderer-independent WGSL validation/source mapping and revisioned transactional effect replacement are complete.
- [x] **H — Asset readiness.** Canonical source assets keep stable keys while revisions/content change.
- [x] **I — Syntax and parser.** Lossless CST, typed AST facade, spans, comments/trivia, incomplete-input recovery, and parser diagnostics are complete.
- [x] **J — Semantic compiler.** Modules, imports, exports, user types, components, properties, callbacks, slots, expressions, repeaters, themes/styles, states, animations, effects, and assets validate incrementally.
- [x] **K — Typed IR.** Every semantic reference is normalized to an explicit ID and the IR carries optional source metadata.
- [x] **L — Release backend.** Reachability/DCE, Rust AOT generation, build integration, typed bindings, assets, and effects produce a release application without development DSL dependencies.
- [x] **M — Live runtime.** Pre-resolved expression bytecode, dynamic components/properties/models, state migration, retained identities, themes, animations, effects, assets, and inspection pass parity fixtures.
- [x] **N — Protocol and `argui dev`.** Versioned transport-independent packages, watcher/compiler service, compatibility checks, transactional commits, and native hot reload work without Rust recompilation.
- [x] **O — `@argui/ui`.** Official components are exposed through the schema and authored in DSL where behavior permits; docs/examples no longer require Rust widget builders.
- [x] **P — Web/mobile transports.** WebSocket browser and host-to-device development transports work while release artifacts remain unchanged.
- [x] **Q — LSP, formatter, AI CLI.** Diagnostics, completion, hover, navigation, references, rename, symbols, semantic tokens, formatting, color presentation, code actions, and JSON schema/check/complete commands are production-ready.
- [x] **R — Remove Subsecond UI hot reload.** Old features, dependency, runtime bridge, gallery page, tests, and documentation are removed only after DSL live parity.

## Verification cadence

During a phase, run only its affected packages with:

```sh
cargo nextest run -p PACKAGE --all-features
cargo clippy -p PACKAGE --all-targets --all-features -- -D warnings
```

Graphical scenarios must run through `./scripts/linux-hidden-display.sh` and
must produce a non-blank capture. After every phase and parity fixture passes,
run exactly once:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```
