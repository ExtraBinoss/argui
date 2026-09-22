# DevTools

`DevtoolsHost<A>` wraps one `Render` component. `DevtoolsApp<M>` wraps a
multi-window `AppModel`. Both preserve the application's assets, events,
animation lifecycle, and renderer configuration while attaching an
`InspectorHandle`.

The tools dock at the bottom or right and give the application the remaining
viewport. Native applications can use `DockMode::Detached`; Web uses the
in-canvas dock. The frontend is built from ordinary Argui widgets.

## Elements and styles

The virtualized tree supports filtering, expansion, keyboard navigation,
selection, and a pointer picker. Hover highlights bounds without mutating the
application tree. Below 640 px, tree and property panes become separate views.

Style overrides cover background, border, paint and layer opacity, overflow,
transform, effects, width, and height. Inputs validate before applying.
Individual properties can be enabled, reset, or removed.

Overrides belong to the inspector and run before layout or paint lowering. They
do not mutate application state or source code. Paint overrides win over the
same authored interaction property; unrelated hover and pressed styles continue
to work. Each property uses its normal paint or layout invalidation path.

Unsupported values remain read-only summaries. Gradients, for example, are not
silently replaced with solid colors. The exact protocol types are
[`StyleProperty` and `StyleValue`](../../crates/argui-inspect/src/lib.rs).

## Theme

The Theme pane edits the standard palette and overlay blur values for the
inspected window. It can preview system, light, or dark mode and export resolved
tokens as JSON. Reset restores the application environment.

Applications using the standard widgets should resolve
`default_theme(&cx.environment())`. Custom themes can read `ThemeSource`
tokens directly. Hard-coded colors cannot become editable tokens automatically.
See [styling](../ui/styling.md#themes).

## Recording

`argui-inspect` owns renderer-independent snapshots and a bounded 300-frame
ring. Records contain CPU stages, draw counts, invalidations, GPU passes,
processed pixels, offscreen textures, and vector/glyph-atlas activity. Glyph
counters include cache hits, raster requests, upload bytes, page evictions,
and the allocated mask/color texture bytes.

GPU durations require timestamp-query support and remain separate from CPU
durations. Pause freezes collection; Clear releases history. Export writes
strict `argui-gpu-trace-v5` JSON; import rejects unknown versions and fields.

Detailed GPU profiling runs only while the Profiling pane is open and unpaused.
Closing the tools stops recording. Selecting a frame freezes its details; Live
follows incoming records.

## Resources

Resources distinguishes:

- process CPU and memory;
- Linux RSS/PSS/private mapping categories;
- known Argui buffer and cache capacities;
- GPU texture-pool and vector-atlas bytes;
- optional device-wide hardware sensors.

These values are different accounting systems and must not be added together.
Known capacities exclude nested payloads, fonts, images, shared styles, hash
tables, and allocator overhead.

Process sampling runs at most once per second while Resources is visible and
unpaused, with one worker request in flight. Browser hosts cannot expose process
totals.

For device sensors, implement `DeviceTelemetryProvider` or enable
`argui-devtools/all-smi` (`argui/devtools-all-smi` through the facade).
`AllSmi` launches the external
[all-smi executable](https://github.com/lablup/all-smi) without a shell and
reports missing tools, timeouts, and reader errors in the panel. Sensor support
depends on the GPU and driver; detecting a device does not guarantee every
metric.

## Cost and verification

`InspectionCache` publishes a new tree only when shallow descriptions or
geometry change. The tree view retains visible rows plus overscan. Resource and
profiling work stops when its pane is inactive.

Focused CPU workloads:

```sh
cargo run -p argui-devtools --all-features --example profiling
cargo run -p argui-widget-gallery --all-features \
  --example resize -- --large-tree
```

Run them without concurrent builds or tests. General measurement rules are in
[performance](../performance/optimizations.md).

Browser UI coverage lives in `crates/argui-devtools/tests/view.mjs`. Native
dock and detached-window coverage lives in
`crates/argui-devtools/tests/app.py`. Run both through the
[private Linux display](linux-testing.md).
