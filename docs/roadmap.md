# Remaining roadmap

Argui already has the native/WASM foundation: `winit`, WGPU surfaces, retained
state, Taffy layout, Cosmic Text shaping and editing, interaction, clipping,
scroll, fixed-row virtualization, animations and physics, transforms, arbitrary
linear/radial gradients, bounded image textures, scoped GPU layers, custom
WGSL, and optional effect presets.

This document tracks only unfinished product work. Detailed invariants stay in
their dedicated documents and completed milestones are not repeated here.

## 0. Integrated DevTools — foundation implemented

- Implemented: optional `argui-inspect` protocol, reusable `DevtoolsHost<A>`,
  resizable bottom dock, virtualized searchable Elements tree, stable selection,
  bounds highlight, reversible typed style switches, bounded frame timeline,
  CPU stages, effect passes, offscreen pixels and texture-pool diagnostics.
- Next: expand/collapse and picker mode; richer authored/resolved style editors;
  per-node/effect cost ranking; optional GPU timestamps; trace export/import;
  keyboard and accessibility hardening.
- The controller/frontend split already keeps a later detached native window
  independent from runtime and renderer internals.

Done when the current popover stall can be isolated from the dock without
console logs, and disabled DevTools preserve the existing idle fast path.

## 1. Text and form controls

- Add multiline editing, selection, vertical caret navigation, scrolling, and
  richer IME composition decoration.
- Add controlled text-input values without losing retained selection or IME
  state.
- Build textarea, checkbox, slider, menu/select, and reusable modal/popover
  behavior from existing primitives.
- Add focus traps, focus restoration, and keyboard navigation for overlays.

Done when complex text remains grapheme-safe and bidi-correct, and every control
shares one behavior on native and web.

## 2. Retained state and large-tree performance

- Add reusable component-local state ownership keyed by stable identity.
- Invalidate layout and paint per subtree instead of rebuilding global outputs.
- Cache intrinsic text and widget measurements with explicit invalidation.
- Extend virtual lists to variable row heights while keeping bounded work and
  memory for million-item data sets.

Done when profiling demonstrates work proportional to the changed/visible
subtree rather than total tree size.

## 3. Input and accessibility

- Normalize touch, multitouch, and gesture input without imposing a gesture
  policy on applications.
- Emit semantic accessibility nodes, roles, labels, values, actions, and focus
  updates for native accessibility APIs and the browser target.
- Verify keyboard-only, screen-reader, reduced-motion, and high-contrast paths.

Done when composed widgets expose semantics without renderer knowledge and the
same Rust tree drives native and browser accessibility.

## 4. Animation ergonomics

- Bind typed animation values directly to transforms, layout properties,
  scroll, caret presentation, and effect parameters.
- Add concise declarations for animated custom-effect parameters while keeping
  raw `Timeline<T>`, spring, decay, and inertia APIs available.
- Define interruption and composition behavior for property-level animations.

The engine, scheduler, keyframes, orchestration, implicit paint transitions,
springs, decay, and inertia are already implemented. This step is API wiring,
not another timing engine.

## 5. GPU hardening

- Add optional GPU timestamp queries where adapters expose them.
- Add deterministic golden images for transforms, gradients, rounded masks,
  liquid glass, refraction, shadows, and custom outer effects.
- Profile representative native and browser scenes before adding more fused or
  specialized pipelines.
- Tune cropped targets, texture-pool limits, and effect quality from measured
  GPU time and memory rather than guesses.

## 6. Optional DSL — last

- Build a separate parser/compiler that lowers into the same public `Element`,
  layout, scoped-effect, and animation APIs used by Rust builders.
- Keep parsing, diagnostics, hot reload, and tooling outside the runtime and
  renderer crates.
- Ensure every DSL feature has an equivalent direct Rust representation.

The DSL starts only after the underlying Rust APIs for transforms, widgets,
state, and accessibility are stable enough to avoid encoding temporary designs.

## Acceptance gate

Every roadmap item must:

- behave through the same application tree on native and WASM;
- preserve the idle/no-effect fast paths;
- include focused CPU tests and GPU tests where pixels matter;
- keep every Rust file below 600 lines;
- pass `./scripts/quality.sh` with at least 85% branches, functions, lines, and
  regions.
