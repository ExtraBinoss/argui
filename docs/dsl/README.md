# Argui DSL implementation checklist

This is the delivery tracker for the [implementation plan](ARGUI_DSL_IMPLEMENTATION_PLAN.md),
not a claim that every phase is finished. A checked item names an implemented path;
an unchecked exit gate still needs verification or work. Keep this file accurate
as the language and examples evolve.

The [Slint-style composition architecture plan](SLINT_COMPOSITION_ARCHITECTURE_PLAN.md)
tracks the composition migration. Its implemented boundary and historical
baseline are recorded in the [composition inventory](composition-inventory.md).
Unchecked gates below still require work or final verification.

## Phase A — Architecture contracts

- [x] Record the ten engine, schema, asset, protocol, and ABI decisions in [`adr/`](adr/).
- [x] Keep the renderer and runtime independent of DSL crates.

## Phase B — Dynamic engine names

- [x] Use owned names for DSL-relevant identities and test dynamic source names.

## Phase C — Retained source identity

- [x] Reconcile compiler-generated site/instance identities across reorder and reload.

## Phase D — Reactive core

- [x] Typed properties, transactions, dependency tracking, and two-way links have crate tests.
- [ ] Recheck the final per-crate coverage gate after implementation settles.

## Phase E — Theme v2

- [x] Typed tokens resolve by mode and can be overridden by custom themes.
- [x] The DSL gallery switches the same token names across light, dark, and accent modes.

## Phase F — Declarative native schema

- [x] Native elements construct from schema IDs and typed values without parser-specific dispatch.
- [ ] Verify the final virtual-list and animation additions against the schema conformance tests.

## Phase G — Shader and effect live readiness

- [x] WGSL validation and diagnostics live in `argui-shader`.
- [x] Revisioned effect replacement has Rust-only tests.

## Phase H — Stable assets

- [x] Stable asset revisions and generic PNG, WebP, and SVG imports work in AOT/live paths.
- [x] Tabler icons use the same asset pipeline and are feature-gated/reachability-pruned.
- [ ] Confirm an edited asset in an interactive `argui dev` session.

## Phase I — Syntax and parser

- [x] Separate syntax/parser crates retain spans, recover from errors, and expose AST/CST.

## Phase J — Semantic compiler

- [x] Multi-file modules, imports, properties, bindings, styles, themes, assets, and effects are checked.
- [ ] Complete and test animation keyframes, easing, and enter/leave/in-out policies from section 13.
- [x] Complete and test virtual-list language semantics and the gallery sidebar.

## Phase K — Typed IR

- [x] Semantic references lower to stable typed identifiers.
- [ ] Verify new animation and virtual-list nodes with IR round-trip and compatibility tests.

## Phase L — Release backend

- [x] `dsl-build` generates a multi-file gallery without a release dependency on the live compiler.
- [ ] Build the final gallery in release mode without warnings and inspect its bundled assets.

## Phase M — Live runtime

- [x] AOT/live rendering and state migration have integration tests.
- [x] AOT/live parity covers every gallery page, button/input/switch/theme
  changes, overlays, slider drags, and private-display pixel captures.

## Phase N — Development service

- [x] `argui dev` watches project/stdlib sources and publishes transactional generations with client acknowledgements.
- [x] A remote live-package test verifies visible UI updates without user input.
- [ ] Recheck the complete interactive gallery and detailed rejection diagnostics.

## Phase O — Standard library migration

- [x] `Button`, `Input`, `Card`, `Badge`, `Separator`, and `Switch` are separate DSL modules.
- [x] The independent gallery uses those modules and separate DSL pages.
- [ ] Continue migrating the remaining public widget catalogue and primary samples to DSL.

## Phase P — Web/mobile live transports

- [ ] Browser and Android/iOS development clients receive host packages and pass parity tests.

## Phase Q — Authoring tools

- [x] Formatter, schema/query CLI, completion, and source-located diagnostics have tests.
- [ ] Run the final cross-platform authoring checks and documentation review.

## Phase R — Retire old UI hot reload

- [ ] Remove Subsecond UI hot-reload features, dependencies, bridge, and docs only after DSL parity.

## Final delivery gate

- [ ] Targeted all-features Nextest runs pass for every changed crate and the gallery.
- [x] Private-display capture is nonblank and the gallery is usable at narrow widths.
- [ ] `./scripts/quality.sh` passes once after implementation, including every 85% coverage gate.
- [ ] Audit the diff, preserve the user's fixture changes, and commit the finished work.
