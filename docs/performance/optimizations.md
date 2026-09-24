# Performance

Argui's performance contract is structural:

- retained UI, layout, text, and paint data survive between frames;
- clean subtrees reuse their cached element descriptions;
- paint-only changes do not run layout or shape text;
- transforms and scroll offsets reuse layout geometry;
- virtual lists mount a bounded visible window;
- an idle application schedules no frame.

These rules are tested. Timing and memory depend on hardware, drivers,
features, viewport, and workload; measure them for the application being
investigated.

## Animate presentation properties first

Transforms and layer opacity are presentation properties: Argui keeps their
layout geometry stable, patches hit-testing and semantic geometry, reuses the
prepared GPU primitives and text, then damages only the old and new visual
bounds. This is the cheapest path for movement, scaling, rotation, and fades.
The renderer applies the same bounded-damage logic through retained blur,
shadow, and custom-effect layers.

Animating width, height, padding, margin, gaps, or flex/grid placement changes
the layout contract and therefore runs Taffy again. Use those properties when
surrounding content must genuinely reflow; do not use an animated margin merely
to move an otherwise independent visual. No application-specific invalidation
or damage bookkeeping is required for either path.

## Keep visual loops out of model rebuilds

A component animation frame is appropriate when application state or rendered
content genuinely changes. Calling `Context::notify` on every display frame
rebuilds and diffs that component and its dependent parents, even if damage
rendering later repaints only a few pixels.

For a purely visual loop, bind a typed `Motion` directly to `Transform`,
`LayerOpacity`, or another retained property. Argui samples the track inside the
UI tree without rebuilding the model. Transform and layer-opacity bindings use
the compositor path; `LayerOpacity` automatically promotes plain content and
does not require an explicit effect layer. This is suitable for spinners and
skeleton pulses. Reduced-motion handling still stops their tracks rather than
leaving an invisible frame loop active.

The animation registry checks inactive tracks without taking their value lock.
Only active tracks are locked and sampled, so a screen can keep many dormant
hover or focus transitions without making an unrelated continuous animation
more expensive.

## Frame-coalesce continuous input

Use `GestureDelivery::FrameCoalesced` when a gesture update invalidates a view,
changes layout, shapes text, or records new paint. It is Argui's portable
equivalent of scheduling visual input work with `requestAnimationFrame`: the
runtime keeps the newest state for each gesture stream and delivers at most one
`Changed` event on each available display frame.

The callback rate is capped by the active display refresh cadence, such as 60,
120, or 144 Hz. It can be lower under load or while a browser tab is backgrounded;
it is not a fixed-rate timer. `Started`, `Ended`, and `Cancelled` remain immediate.
For pans, the delivered sample contains the latest position, total displacement,
and velocity plus the sum of every delta accumulated since the previous frame.

```rust
Interaction::default().gestures(
    GestureSet::EMPTY.pan(
        PanGesture::default()
            .immediate()
            .capture(GestureCapture::OnPress)
            .delivery(GestureDelivery::FrameCoalesced),
    ),
)
```

Prefer `Immediate` only when every raw input sample is application data and
discarding intermediate samples would change the result. Frame coalescing bounds
callback frequency, but each callback must still fit within the frame budget.
Keep stable keys, prefer transform or paint changes over layout, and defer durable
work to the final commit or release event.

## Measure the TSX gallery

Build the Solid or React bundle first, then compile the native QuickJS host with
that bundle embedded. Keep the same Cargo profile, GPU driver, viewport, and
framework when comparing revisions. On Linux, read `VmRSS` and `RssAnon` from
`/proc/<pid>/status` separately; thread rows in a process monitor share the
process address space and must not be added together. GPU resident memory is a
separate measurement.

During development, the gallery script watches the TSX bundle and updates the
running app:

```sh
./scripts/gallery-hot-reload.sh desktop solid
```

The hot reload regression probe should repeat enough edits to detect a rising
trend after warm-up. A single before/after sample includes allocator and GPU
cache warm-up. The development renderer waits for submitted GPU work after each
frame to bound retired resources across repeated reloads; release builds do not
use that wait.

## Measurement rules

- Alternate reference and changed runs after the same warm-up.
- Do not build, run coverage, or launch unrelated tests while sampling.
- Verify that graphical windows continue presenting.
- Compare geometry checksums, retained-node counts, and layout counts before
  comparing time.
- Report regressions as well as improvements. Do not combine release and
  development profiles.
- Distinguish CPU time, frame interval, input-to-display latency, retained heap,
  RSS, anonymous RSS, and GPU memory.

Performance changes still pass the behavioral [quality gate](../contributing/code-quality.md).
Graphical profiles and captures use the
[private Linux display](../contributing/linux-testing.md).
