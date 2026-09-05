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
- vector-atlas entries, cache hits, rerasterizations and allocated bytes;
- GPU timestamp durations when the adapter exposes timestamp queries.

Selecting a frame shows its stage waterfall and the most expensive nodes/layers.
The summary ranks costs by total time, peak time, pixels, allocations and
invalidation count. CPU timings must not be labelled GPU time; unsupported GPU
timestamps are displayed as unavailable.

Recording uses a fixed ring of 300 frames and is active only while the dock is
open and not paused. Pause freezes the history, clear releases it, and export
writes strict `argui-gpu-trace-v2` JSON. Unknown versions, fields and enum
values are rejected during import. The same data-only document can be compared
across Linux, Windows, macOS and WebGPU without platform-specific parsing.

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

## Profiling presentation and refresh

Profiling keeps its commands outside the scroll regions. At 760 logical pixels
and above, the overview and selected detail view are side by side; narrower
panels switch between Overview, GPU passes, and Frame details. GPU passes use
the shared VList widget, with aligned name, duration, and timing columns.

The frame history uses actual numeric columns, not whitespace alignment. Each
pane has one scrollport: VList scrolls its measured header together with the
virtual rows. Stable scrollbar gutters reserve space beside the content. GPU
bar position indicates start time and bar width indicates duration on the same
scale; a short pass is intentionally drawn as a short bar.

The frame graph has 60 retained slots on a fixed 0–50 ms scale (larger peaks are
clipped visually, not in recorded values). Its 120 ms paint-only transitions
interpolate between 100 ms frontend samples; they do not animate layout heights
or alter trace measurements. Heavy frame details refresh twice per second and
reuse their element subtree between samples. Selecting a recorded frame freezes
the displayed snapshot; Live returns to the current recording.

Detailed renderer profiling is enabled only in the Profiling tab, and disabled
in Elements, while paused, or when tools are closed. Tree inspection and CPU
history remain independent of this GPU measurement switch.

The frontend CPU workload can be measured independently of GPU query and
presentation costs (run without concurrent builds or tests):

```sh
cargo run -p argui-devtools --all-features --example profiling
```

It exercises closed, Elements, and live Profiling states with 80 simulated GPU
passes and changing timings. It follows retained layout-versus-paint
invalidation and discards 40 warm-up ticks. These CPU results are not an
end-to-end FPS measurement.

## Scroll inspection cost

`InspectionCache` compares shallow inspectable node descriptions and their
geometry before allocating a snapshot. Changes confined to an
`inspectable(false)` subtree do not republish the application tree. Application
content, style, ancestry, clipping, and portal changes still invalidate it.

The widget-level `TreeViewCache` retains only visible rows plus overscan. It
reuses rows while scrolling and invalidates them when data, expansion, selection,
theme, or vector icons change. DevTools node icons and disclosure arrows use
embedded vectors rather than font glyphs.

To reproduce the CPU inspection workload:

```sh
cargo run -p argui-widget-gallery --all-features --example resize -- --large-tree
```

On the development machine, a 10,000-node unchanged tree measured 2.9 ms p95
with the cache versus 26.6 ms for allocating a complete snapshot (30 samples,
optimized development profile). This isolates inspection CPU cost; it does not
measure end-to-end scrolling, GPU time, or prove 60 FPS. The same example without
`--large-tree` measures the composition resize workload.

## Acceptance gate

- Same inspected tree and DevTools UI on native and WASM.
- No WebView, HTML inspector, TypeScript frontend, or renderer-specific widgets.
- No allocation or frame request from detached instrumentation.
- DevTools never appears inside its own default Elements hierarchy.
- Style overrides are reversible and never mutate application-owned state.
- CPU and GPU measurements are named accurately.
- All Rust files remain below 600 lines and `./scripts/quality.sh` stays above
  85% for branches, functions, lines, and regions.
