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
bounds immediately; hovering does not change selection or rebuild the app tree.
Typing from the tree or panel focuses **Filter elements** and preserves the first
character. Ctrl/Cmd+F focuses and selects the filter; property inputs keep their
own keyboard input. The divider between the tree and properties supports dragging
and arrow keys. Below 640 px, a back button switches between these panes.
The picker selects application nodes under the pointer. Inspector
elements are excluded from the application tree by default.

The current override API covers background, border, opacity, overflow,
transform, layer, effects, width and height. Available numeric fields can be
edited with validation; incomplete input preserves the last valid value. Width
and height expose Auto/px/% modes, opacity has a slider, overflow has choices,
and background/border colors use the reusable ColorPicker. Each property has an
enable control and an independent reset. Unsupported values remain summaries. This is not an
editor for every authored style property. The exact types live in
[`StyleProperty` and `StyleValue`](../../crates/argui-inspect/src/lib.rs).

Overrides are owned by the inspector and applied before layout/paint lowering;
they never mutate application state. Clearing them restores authored values.
Each property selects the appropriate invalidation path.

## Live theme

**Theme** previews the inspected window's 18 palette colors and two overlay blur
radii. Its editors offer the same color picker, validation and individual resets.
App theme/Light/Dark changes the preview scheme; **Reset theme** restores the
application's environment. **Copy JSON** exports the resolved palette and scheme.
Edits remain in this inspector session and do not rewrite source files.

Consumers must resolve `shadcn(&cx.environment())`, or read `ThemeSource` tokens
in their own theme implementation. Hard-coded colors cannot become theme tokens
automatically. The preview propagates to mounted child entities and is scoped to
the inspected window; other windows and the DevTools palette retain their own
environment. See [Themes](../ui/styling.md#themes).

## Recording and traces

`argui-inspect` owns renderer-independent snapshots and bounded frame records.
The runtime's inspector handle is optional; engine crates do not depend on the
DevTools frontend. The controller retains the latest tree rather than a history
of complete trees or GPU resources.

The 300-frame ring records CPU stages, drawing counts, invalidations, GPU passes,
processed pixels, offscreen texture use and vector-atlas activity. GPU timings
require adapter timestamp support; CPU timings are reported separately.
Pause freezes recording, clear releases history, and export produces strict
`argui-gpu-trace-v3` JSON. Import rejects unknown versions, fields and enum values.

Detailed renderer profiling runs only in the open, unpaused Profiling tab.
Tree inspection and CPU history are independent of this switch. Closing the
tools stops recording. A selected frame freezes the detail snapshot; Live follows
current records. Refreshing the frontend does not change trace measurements.

The graph has 60 retained slots, samples at 100 ms and shows a fixed 0–50 ms
scale. Larger peaks remain in the records. Paint-only transitions interpolate
the graph; heavy details refresh twice per second and reuse cached subtrees.
GPU pass rows are virtualized. Responsive panes keep commands outside scrollports.

## Resources and hardware sensors

**Profiling → Resources** shows CPU stages as milliseconds and fractions of the
recorded CPU total. Process CPU is separate: 100% means one logical CPU fully
occupied, and its first sample is unavailable until an interval exists.

Native process measurements use `sysinfo`, refreshing only this PID's CPU and
memory, with thread enumeration disabled. Linux additionally reads `smaps` for
RSS, PSS, private resident pages, and categories that sum to RSS: heap, anonymous
allocations, stacks, files/libraries, device mappings and kernel mappings. Expand
a category for its largest mappings. This does not attribute allocator arenas
to individual widgets. Other native OSes expose process totals; mapping details
are currently Linux-specific. Browser process totals and physical sensors are
unavailable through these APIs.

Known Argui allocations are separate capacity counters: traversal columns,
layout metadata, Taffy caches, geometry, output buffers and paint commands. They
work in native and Web builds, require no tree traversal, and exclude nested
payloads, shared styles, fonts, images, hash tables and allocator overhead.
They are not a decomposition of RSS. Texture pool and vector atlas allocations
are shown separately as GPU bytes; never add these to process RAM.

Collection runs at most once per second, only while Resources is open and
unpaused. Native collection uses a lazy worker, a 512 KiB stack and channels of
capacity one; there is at most one request in flight. Closing the tools, pausing
or selecting another pane stops new requests. An in-flight request may finish.
The panel reports collection time so its own cost is visible. Linux details
retain at most 12 rows per category and parsing has an 8 MiB input budget.

GPU telemetry is optional and device-wide. Implement
`DeviceTelemetryProvider: Send` and pass it to `.device_telemetry(provider)` on
either wrapper. Return `None` for unavailable readings and bound blocking work.
This interface does not require any vendor SDK in the renderer/runtime.

The `argui-devtools/all-smi` feature provides `telemetry::AllSmi`; the facade calls
it `devtools-all-smi`, and the gallery calls it `all-smi`. Install the
[separate all-smi executable](https://github.com/lablup/all-smi), or configure a
path with `AllSmi::new(path)`. After compiling the feature, explicitly enable
its sensors in Resources. The adapter requests schema-1 JSON snapshots with no
shell, server or elevated permissions; default timeout is two seconds and
stdout is limited to 2 MiB. Missing binaries, reader errors and timeouts appear
in the panel. Keeping this an external adapter avoids linking all-smi's broader
network/server and accelerator dependencies into Argui.

Upstream supports Linux, Windows and macOS, with sensors depending on hardware
and drivers. This integration was exercised on Linux with all-smi 0.26.1.
On this Intel Arc/Meteor Lake machine it detected the GPU but returned zeros
for every sensor. Such detection-only records show unavailable primary values;
Driver details retain raw values and reader warnings. An all-zero record cannot
distinguish unsupported sensors from a powered-down device. An idle 0% remains
valid when other nonzero readings exist. Windows/macOS execution has not been
validated here; compiling the feature does not guarantee every GPU sensor.

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
`crates/argui-devtools/tests/view.mjs` exercises real browser input and checks
rendered pixels, including the sRGB color pad and live theme. For the native
gallery, run `crates/argui-devtools/tests/app.py` through the private X11 backend;
its optional `--seconds` samples CPU/RSS without instrumenting application code.
General CPU and memory studies are in [performance](../performance/optimizations.md).
