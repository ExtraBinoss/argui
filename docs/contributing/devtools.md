# DevTools

`DevtoolsHost<A>` wraps a `Render` component; `DevtoolsApp<M>` wraps a multi-window
`AppModel`. Both expose an `InspectorHandle` and retain the inspected application's
assets, event routing and animation lifecycle. Use `configure_renderer` to add
the dock's effects while preserving application shaders.

The dock opens at the bottom or right, resizes with a splitter and leaves the
application its remaining viewport. `DevtoolsApp` also supports
`DockMode::Detached` on native platforms. Detachment keeps the dock visible until
the tools window is ready; an opening failure returns to the dock. Web uses
the in-canvas dock. The frontend consists of ordinary Argui widgets.

## Elements and styles

The virtualized tree supports search, expand/collapse, stable selection and
keyboard navigation. Hovering or selecting a row highlights the corresponding
bounds. The picker selects application nodes under the pointer. Inspector
elements are excluded from the application tree by default.

The current override API covers background, border, opacity, overflow,
transform, layer, effects, width and height. Available numeric fields can be
edited; other values are displayed as choices or summaries. This is not an
editor for every authored style property. The exact types live in
[`StyleProperty` and `StyleValue`](../../crates/argui-inspect/src/lib.rs).

Overrides are owned by the inspector and applied before layout/paint lowering;
they never mutate application state. Clearing them restores authored values.
Each property selects the appropriate invalidation path.

## Recording and traces

`argui-inspect` owns renderer-independent snapshots and bounded frame records.
The runtime's inspector handle is optional; engine crates do not depend on the
DevTools frontend. The controller retains the latest tree rather than a history
of complete trees or GPU resources.

The 300-frame ring records CPU stages, drawing counts, invalidations, GPU passes,
processed pixels, offscreen texture use and vector-atlas activity. GPU timings
require adapter timestamp support; CPU timings are reported separately.
Pause freezes recording, clear releases history, and export produces strict
`argui-gpu-trace-v2` JSON. Import rejects unknown versions, fields and enum values.

Detailed renderer profiling runs only in the open, unpaused Profiling tab.
Tree inspection and CPU history are independent of this switch. Closing the
tools stops recording. A selected frame freezes the detail snapshot; Live follows
current records. Refreshing the frontend does not change trace measurements.

The graph has 60 retained slots, samples at 100 ms and shows a fixed 0–50 ms
scale. Larger peaks remain in the records. Paint-only transitions interpolate
the graph; heavy details refresh twice per second and reuse cached subtrees.
GPU pass rows are virtualized. Responsive panes keep commands outside scrollports.

## Cost and reproduction

`InspectionCache` compares shallow descriptions and geometry before allocating
a snapshot. Changes confined to an `inspectable(false)` subtree do not republish
the application tree. `TreeViewCache` retains visible rows plus overscan and the
active row, reusing them until data, expansion, selection, theme or icons change.

Run CPU workloads without concurrent builds or tests:

```sh
cargo run -p argui-devtools --all-features --example profiling
cargo run -p argui-widget-gallery --all-features --example resize -- --large-tree
```

The first compares closed, Elements and live Profiling states with 80 simulated
GPU passes, discarding 40 warm-up ticks. The second measures a 10,000-node
unchanged inspection tree. A recorded development-machine comparison measured
2.9 ms p95 with caching versus 26.6 ms with full snapshot allocation, over 30
samples in the optimized development profile. This isolates inspection CPU
cost; it is not a scrolling FPS or GPU measurement.

For graphical checks, use the [private Linux display](linux-testing.md).
General CPU and memory studies are in [performance](../performance/optimizations.md).
