# Argui DevTools

The protocol, dock host, searchable virtualized tree, selection highlight,
typed style switches, and bounded profiling timeline are implemented. The build
sequence below remains the design contract; unfinished hardening is tracked in
the main roadmap.

The DevTools are the next milestone, before the remaining product roadmap. They
must inspect the same native/WASM engine they are built with, remain optional,
and never introduce work when disabled.

## Product shape

The first frontend is a resizable bottom dock, similar to browser developer
tools. Opening it reduces the application's viewport instead of covering it:

```text
+-------------------------------------------------------+
| inspected application                                |
|                                                       |
+================ draggable splitter ==================+
| Elements | Profiling                                  |
|                                                       |
| DevTools                                             |
+-------------------------------------------------------+
```

The dock supports open/close, height dragging, keyboard toggle, and a minimum
application viewport. Its state is retained without rebuilding the inspected
application. A later detached window must reuse the same inspector model and UI
without changing engine instrumentation.

## Crate boundaries

Two small crates keep dependencies acyclic:

- `argui-inspect`: renderer-independent snapshots, event records, bounded frame
  history, style overrides, and the inspector controller. Engine crates publish
  data into this protocol but never depend on the DevTools UI.
- `argui-devtools`: composed Argui widgets for the dock, tree, style sidebar,
  profiling timeline, and selection highlight. It consumes `argui-inspect` and
  depends on public Argui UI APIs only.

The runtime owns an optional inspector handle. With no handle attached, all
instrumentation paths must reduce to a predictable branch and allocate nothing.
The inspected application and the DevTools subtree receive distinct scope tags
so the tree view excludes itself by default and profiling can report its own
overhead separately.

## Inspection protocol

Snapshots use stable `NodeId` values and contain only immutable, bounded data:

- parent/child order, key, element kind, and semantic label;
- logical and transformed bounds, clips, z-index, visibility, and scroll state;
- authored style plus resolved layout, paint, text, image, interaction, layer,
  filter, and animation values;
- invalidation reason: none, paint, text preparation, subtree layout, or full
  layout;
- display-list ranges and effect-layer identity for cost attribution.

The controller retains the latest tree snapshot and a bounded frame ring. It
must not retain complete historical trees, glyph buffers, images, GPU handles,
or texture contents.

## Elements tab

The left pane is a virtualized hierarchy with expand/collapse, text filtering,
stable selection, breadcrumbs, and keyboard navigation. Hovering or selecting a
row draws a non-interactive highlight over the corresponding application bounds.

The right style pane groups properties by responsibility:

- layout: size, min/max, flex/grid, gap, padding, margin, position;
- paint: background, gradient, border, radii, opacity and clipping;
- text and image presentation;
- transforms, z-index, interaction and scroll;
- layers, shadows, filters, backdrop filters, blend modes and custom effects.

Every authored property can be enabled or disabled. Overrides live in the
inspector, never mutate application state, and are applied after the Rust tree is
built but before layout/paint lowering. Clearing the inspector restores the exact
application output. Each override declares whether it requires paint, text, or
layout invalidation; toggling a color must never trigger Taffy or text shaping.

The first version edits booleans, numbers, colors, lengths, enums, and complete
effect entries. Rich gradient stops and custom WGSL parameters can use dedicated
editors after the generic path is proven.

## Profiling tab

Profiling is explicit and bounded. A frame timeline records:

- frame interval and missed-frame threshold;
- model update, tree diff, layout, text preparation and repaint CPU time;
- renderer command-encoding CPU time;
- draw batches, quads, glyphs, images and display-list commands;
- effect layers, named filter passes, processed pixels and downsample factors;
- offscreen texture acquisitions, reuse, allocation, eviction, current bytes and
  peak bytes;
- GPU timestamp durations when the adapter exposes timestamp queries.

Selecting a frame shows its stage waterfall and the most expensive nodes/layers.
The summary ranks costs by total time, peak time, pixels, allocations and
invalidation count. CPU timings must not be labelled GPU time; unsupported GPU
timestamps are displayed as unavailable.

Recording defaults to a fixed ring, initially 300 frames. Pause freezes the
history, clear releases it, and export writes a versioned data-only trace. WASM
export uses a browser download; native export uses a caller-selected path.

## Build sequence

### 1. Protocol and measurements

- Create `argui-inspect` with tree/frame snapshots and a bounded recorder.
- Convert the current ad-hoc `AnimationProfile` and `RenderProfile` into the
  shared protocol.
- Add per-stage CPU timing, invalidation reasons, texture allocation/eviction
  counters, and stable effect-layer identifiers.
- Preserve the existing `ARGUI_PROFILE=1` text output as a thin protocol sink.

Checkpoint: headless tests prove bounded memory, stable identities, correct
cost attribution, and zero records when inspection is detached.

### 2. Dock host

- Create `argui-devtools` and a `DevtoolsHost<A>` wrapper around any `Render` component.
- Compose application viewport, splitter and dock with normal Argui layout.
- Implement open/close, resizing, minimum sizes and keyboard toggle.
- Keep native and WASM launch code unchanged apart from opting into the wrapper.

Checkpoint: one shared showcase opens the dock on native and WASM, and resizing
the sheet updates the application viewport without overlaying or duplicating it.

### 3. Tree inspection and picker

- Publish a snapshot after committed layout/paint.
- Render the hierarchy as a virtualized list.
- Add expand/collapse, search, selection, breadcrumbs and bounds highlighting.
- Add a picker mode that selects the topmost hit-tested application node while
  the DevTools overlay remains pointer-transparent.

Checkpoint: inspecting the million-row demo creates rows only for the visible
tree window and selected bounds follow scroll and transforms exactly.

### 4. Style inspection and overrides

- Show authored and resolved values in responsibility groups.
- Add enable/disable and primitive editors through a typed override map.
- Apply overrides at the tree-to-layout/paint boundary.
- Classify each override as paint, text or layout work and display that reason.

Checkpoint: disabling a filter removes its GPU passes; disabling a color only
repaints; disabling a layout property recomputes layout; reset restores the
original tree byte-for-byte at the inspection boundary.

### 5. Profiling frontend

- Build the frame timeline, pause/clear controls and stage waterfall.
- Add expensive-node/effect rankings and texture-pool memory charts.
- Highlight missed frames and texture churn rather than merely dumping averages.
- Add optional WGPU timestamp queries with a clear unsupported state.

Checkpoint: the current popover scene identifies its layer passes, processed
pixels, texture reuse and frame spikes without console logs.

### 6. Hardening and detached-window seam

- Add keyboard navigation, focus containment and accessible labels.
- Measure and display DevTools overhead separately from application cost.
- Add versioned trace export/import and deterministic inspector tests.
- Define a local transport trait between controller and frontend; keep the dock
  implementation direct, then use that seam for a later native window.

Checkpoint: closing DevTools returns to the same idle fast path, recording stays
within its configured memory bound, and the frontend contains no renderer or
application-specific code.

## Acceptance gate

- Same inspected tree and DevTools UI on native and WASM.
- No WebView, HTML inspector, TypeScript frontend, or renderer-specific widgets.
- No allocation or frame request from detached instrumentation.
- DevTools never appears inside its own default Elements hierarchy.
- Style overrides are reversible and never mutate application-owned state.
- CPU and GPU measurements are named accurately.
- All Rust files remain below 600 lines and `./scripts/quality.sh` stays above
  85% for branches, functions, lines, and regions.
