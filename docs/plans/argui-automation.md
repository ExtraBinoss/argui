# Cross-platform Argui automation plan

## Goal

Run the same TSX Argui app and TypeScript test without a visible window on
Linux, macOS, and Windows. The CLI should drive common inputs, inspect the UI,
save screenshots, and monitor runtime performance throughout the test. Mobile
and browser automation are later work. The public commands and test API must not
depend on X11, Wayland, a desktop screen recorder, or OS-specific shell tools.
Application rendering and callbacks continue through the existing
`NativeBridge` and `argui-host` transaction/event contract. Do not introduce a
second JSON protocol for describing or mounting the UI.

## Starting point before implementation

- `argui check` can validate a native TSX app without opening a window through
  the QuickJS host's `ARGUI_VALIDATE_ONLY` path. It does not render frames or
  execute interaction tests.
- `argui-inspect` already exposes a retained tree with node keys, summaries,
  bounds, visibility, and interactivity. Use it for assertions and selectors.
- `argui-render` already renders some effects into offscreen textures, and its
  tests read texture pixels back from the GPU. Its full-scene entry point is
  still `SurfaceRenderer`, which owns a window surface. Extract a render target
  shared by window and offscreen rendering before promising headless images.
- Native input translation and scheduling currently live beside the Winit
  event loop. A true headless runner must drive the runtime below Winit, rather
  than create an invisible OS window.
- The existing Linux Wayland capture script remains useful for real-window
  integration checks; it is not the automation backend.

## Public interface

Use `argui test <app-path> <test-file> --out <directory>` for the full
workflow and `argui screenshot <app-path> --out <file.png>` for a one-frame
shortcut. `<app-path>` selects the app just as `argui build` and `argui dev`
do; it is optional when run from one app directory. The CLI builds the app and
test bundles, launches the headless host, and returns a nonzero status
on a failed assertion, timeout, render error, or missing dependency. Errors
name the failing step and suggest the next fix in English.

The test lives in a separate `.test.ts` or `.test.tsx` file, imports the real
TSX application, and uses a typed test API. Keep automation calls out of
production components. Use `.tsx` only when the test itself needs JSX. This
API is new test tooling, not another UI representation or transaction format.
Its implementation calls the existing runtime event and inspector APIs in
process. Proposed example for the generated counter:

```ts
import { mountGallery } from '../src/main'
import { defineArguiTest } from '@argui/test'

export default defineArguiTest({
  app: mountGallery,
  viewport: { width: 800, height: 600, scale: 1 },
  async run(ui) {
    await ui.getById('increment').click()
    await ui.expectText('Count: 1')
    await ui.screenshot('counter.png')
  },
})
```

The first version needs a small set of everyday actions. Proposed calls:

```ts
await ui.getById('items').scroll({ x: 0, y: 480, unit: 'pixels' })
await ui.getById('context-target').click({ button: 'right' })
await ui.getById('search').fill('Argui')
await ui.keyboard.press('Enter')
await ui.getById('card').dragTo(ui.getById('drop-zone'))
await ui.screenshot('after-interaction.png')
await ui.expectPerformance({ p95FrameTimeMsBelow: 16.67 })
```

`scroll` injects an Argui wheel/trackpad delta at the selected element's
position; `click({ button: 'right' })` sends a secondary-button press and
release through the same pointer path as a real mouse.
Performance assertions are opt-in; every run records diagnostics even when no
threshold is set. A frame budget of 16.67 ms corresponds to 60 Hz, not a
guarantee that every operating system or CI runner can sustain 60 fps.

The test and app execute in one QuickJS
session so the real Solid/React callbacks and `NativeBridge` deliveries are
exercised. Test methods are thin bindings to the Rust automation driver; they
must not serialize an alternate component tree or duplicate the host schema.
Bun can build/check the test file, but `bun:test` does not run inside QuickJS.
An awaited test action should yield to the host: the host applies the input,
processes resulting commits and event deliveries, then resolves the test
promise. A blocking callback from QuickJS into Rust would deadlock this flow.

Selectors should prefer stable authored IDs/keys. Allow coordinates as a
fallback for canvas content without a semantic node. Implement click with a
left or right button, scroll, fill, key press, and drag by composing the
existing platform-neutral pointer, wheel, keyboard, and text events. This
keeps the API small while covering common interactions. The first acceptance
tests should cover a counter click, list scroll, context click, field entry,
and drag. Add touch, IME composition, complex shortcuts, and other gestures
only when a concrete application test needs them. Define clear timeouts and
cancellation for async application work. `ui.wait(ms)` advances the app
scheduler, animations, and callbacks. `ui.pause()` and `ui.resume()` freeze and
restore application time around captures; `ui.sleep(ms)` spends bounded wall
time without advancing the app. Prefer semantic assertions when elapsed time
is not part of the test. A user-controlled virtual clock can be added when
animation tests require a chosen frame timestamp.

The windowless driver addresses its single viewport with the stable `main`
window key. `ui.window('main').resize(...)` recomputes and renders it, recording
layout, surface, and renderer phases. Native `setWindowPosition` and
`getWindowInfo` use the same stable key; absolute position depends on the OS.
The windowless driver rejects position requests because it owns no OS window.

## Implementation sequence

1. **Headless application driver.** Add a focused `argui-automation` crate for
   the driver, assertions, and artifact report. Keep
   `argui-runtime`, `argui-ui`, and `argui-render` independent of the CLI and
   TSX/DSL layers. Give the runtime a platform-neutral driver that accepts
   host commits, injects UI/input events, advances time, settles pending work,
   and exposes the inspector tree. The QuickJS host supplies the app and test
   bundles, using the existing bridge for UI transactions and deliveries.
   First acceptance check: the generated counter app can be clicked and its
   changed text asserted on all three desktop operating systems without a
   display server, window, or GPU.
2. **Full-scene offscreen render target.** Separate the renderer's device,
   resources, and passes from its presentation surface. Render into a texture
   with the requested physical size and scale, using a WGPU adapter that does
   not require a compatible surface. Read back with correctly aligned buffer
   rows, normalize the chosen pixel format/color treatment, and encode PNG.
   Reuse the same paint/effect paths as a visible window. Report adapter/device
   failures clearly; do not silently output a blank image. Acceptance check:
   a known counter frame yields a nonblank 800×600 PNG on Linux, macOS, and
   Windows with no visible window.
3. **TSX test API and CLI.** Add a typed `@argui/test` package for the
   headless driver's selectors, common input actions, assertions, and PNG
   capture.
   Bind it to the QuickJS test session without changing the app's
   `NativeBridge` ABI. Route `argui test` and `argui screenshot` through the
   existing app-path, build, and prerequisite checks. Produce a machine
   readable `report.json` containing step outcomes, durations, adapter info,
   viewport, artifact paths, and failure details. Stream short human-readable
   progress and end with a useful failure message. Keep exit codes stable.
   The report is an output artifact, not an input protocol.
4. **Continuous performance monitoring.** Start sampling before the app mounts
   and stop after the last assertion. Record the host/app PID, and child PIDs
   when the runner starts child processes. Sample per-process CPU and resident
   memory at a bounded interval across the entire run, plus peak memory. Use
   Argui's existing frame records for frame CPU/render time and actual frame
   intervals; report frame count, p50/p95/p99/max, and how many frames exceed
   a configurable budget (16.67 ms for 60 Hz). Store timestamped samples and
   a summary beside the screenshots. Distinguish app startup, interaction,
   idle, and teardown so a CPU spike can be tied to an action. Define CPU units
   clearly (100% equals one fully used logical core), and explain missing
   samples or unsupported counters rather than showing zero. Keep the process
   sampler behind a desktop-only `argui-automation` feature if its cross-OS
   dependency would otherwise affect mobile/WASM; enable it in the desktop
   CLI by default. Prefer one maintained cross-platform crate after checking
   its Linux, macOS, and Windows support; avoid OS-specific shell commands.
   Use an opt-in, nested metric trace for action, model, geometry, paint,
   renderer CPU, and readback spans. Keep frame diagnostics and the raw trace
   in the report so tools can correlate slow frames with actions and inspect
   new phase names without changing the report schema. GPU pass times require
   adapter timestamp queries and must remain nullable. Link the automation
   driver only through a Cargo feature; the ordinary release host must not
   enable that feature or collect these development metrics.
   Performance thresholds fail a test only when explicitly requested in TS.
5. **Cross-platform CI.** Run the same counter TSX test on Linux, macOS, and
   Windows, verify assertion results and nonblank captures, and retain the
   report/artifacts on failure. Verify that PID, CPU, memory, and frame samples
   are present for a sufficiently long test on all three systems. Document a
   software-adapter option where available, but never treat the mere presence
   of WGPU as proof that every CI
   machine has a usable adapter. Keep separate real-window smoke tests for
   event-loop and platform integration.

The first usable release ends here. Later, when needed, add deterministic
frame sequences for screencaps, optional video encoding, image-diff baselines,
virtual-time animation tests, and less common input gestures. Prefer semantic
assertions for routine tests because fonts, GPUs, and rasterization can differ
across systems.

## Design constraints

- A single TSX test must produce the same input sequence regardless of OS.
  The engine, not the desktop compositor, supplies its screenshots.
- A true windowless WGPU target is preferred on each desktop OS. If Linux
  needs a display-backed fallback for a specific backend, an isolated X11
  display is acceptable; it must not become the public test API or a macOS/
  Windows dependency. Document which CI path is actually exercised.
- Behavioral tests must still work when no GPU adapter is available. A
  screenshot step needs a GPU adapter and must report its absence precisely.
- A successful `waitFor` or idle check must be tied to host commits, pending
  work, animation state, and frame completion; a fixed delay cannot establish
  that the UI is ready.
- Save artifacts only inside the selected output directory. Reject unsafe
  capture paths and cap dimensions.
- Preserve one rendering implementation for visible and offscreen targets so
  screenshot behavior exercises the real Argui engine.
- Follow repository quality rules during implementation: focused crate tests
  with `--all-features`, cross-platform build checks, and the full quality
  gate exactly once before a commit.

## What this does not claim yet

The current `ARGUI_VALIDATE_ONLY` mode proves bundle/transaction validity,
not visual fidelity. Offscreen text, blur, WebView content, native menus, and
other OS-owned surfaces each need explicit support or an honest unsupported
error; the first milestone should specify which are present in the generated
counter app. Video compression is separate from capturing a deterministic
frame sequence. The automation API should make these limits visible in its
report rather than silently omit content.

## Technical references

- [Winit `EventLoop` documentation](https://docs.rs/winit/0.30.13/winit/event_loop/struct.EventLoop.html) describes platform display initialization, which is why the headless driver bypasses Winit.
- [WGPU adapter request options](https://docs.rs/wgpu/30.0.1/wgpu/type.RequestAdapterOptions.html) allow adapter selection without a compatible surface and document that a fallback adapter is not guaranteed.
- [WGPU texture-to-buffer copy](https://docs.rs/wgpu/30.0.1/wgpu/struct.CommandEncoder.html), [buffer mapping](https://docs.rs/wgpu/30.0.1/wgpu/struct.Buffer.html), and [copy row alignment](https://docs.rs/wgpu/30.0.1/wgpu/constant.COPY_BYTES_PER_ROW_ALIGNMENT.html) describe the capture readback path.
