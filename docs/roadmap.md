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
- Implemented: picker mode, per-pass GPU timestamps, chronological GPU
  waterfall, expensive-pass ranking, adapter capabilities, strict
  `argui-gpu-trace-v1` export/import, regional damage counters and retained
  static-layer reuse.
- Next: expand/collapse; richer authored/resolved style editors; keyboard and
  accessibility hardening; detached native window transport.
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

- [x] Add reusable component-local state ownership keyed by stable identity.
- [x] Skip shared COW subtrees during tree and Taffy reconciliation.
- [x] Retain static paint fragments per subtree and patch GPU buffers by changed ranges.
- [x] Cache intrinsic text measurements with explicit invalidation.
- [x] Extend virtual lists to variable row heights while keeping bounded work and
  memory for million-item data sets.
- [x] Add deterministic complexity tests and a native/WASM performance showcase.

Done when profiling demonstrates work proportional to the changed/visible
subtree rather than total tree size.

## 3. Input and accessibility — foundation implemented

- [x] Normalize mouse, touch and pen into stable pointer events with multitouch
  identity and an opt-in tap/pan/pinch/rotation gesture arena.
- [x] Emit incremental semantic nodes, roles, labels, values, actions and focus
  updates through AccessKit on native targets and real DOM controls on Web.
- [x] Detect reduced motion and high contrast, support explicit application
  overrides, and keep semantic-only updates off the GPU and layout paths.
- Next: platform screen-reader audits and automated browser accessibility tests.

Done when composed widgets expose semantics without renderer knowledge and the
same Rust tree drives native and browser accessibility.

## 4. Animation ergonomics — implemented

- [x] Bind typed animation values directly to transforms, paint, layout,
  scroll, caret presentation, and effect parameters.
- [x] Add concise declarations for animated custom-effect parameters while keeping
  raw `Timeline<T>`, spring, decay, and inertia APIs available.
- [x] Define interruption and composition behavior for property-level animations.

`Motion<T>` is retained independently from application rebuilds. One generic
`Element::bind` accepts typed property markers, and the active registry
deduplicates shared motions while preserving paint, scroll, and incremental
layout invalidation. Retargeting starts at the presented value; springs retain
velocity; replace/add/accumulate composition has stable priority ordering.

## 5. GPU hardening

- [x] Add non-blocking GPU timestamp queries where adapters expose them.
- [x] Correlate CPU stages, GPU passes, stable layer identities, processed
  pixels, offscreen memory and damage in one bounded frame history.
- [x] Add an explicit typed effect registry with opt-in preset families,
  multi-pass shaders and adapter-bounded parameter storage.
- [x] Add static offscreen-layer reuse and quality-controlled spatial-effect
  downsampling without changing the default rendering quality.
- Add deterministic golden images for transforms, gradients, rounded masks,
  liquid glass, refraction, shadows, and custom outer effects.
- Record comparable state/performance-showcase traces on Linux, Windows, macOS
  and WebGPU before changing pass fusion or quality defaults.

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
