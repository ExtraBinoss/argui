# Native automation and performance reports

`argui test <app-path> <file.test.ts> --out <directory>` runs a real Solid or
React application in the windowless QuickJS host. It needs no display server.
The test bundle contains the app, and the optional automation and metrics code
is linked into the test host rather than the ordinary release host.

For the Solid gallery, run:

```sh
argui test apps/gallery tests/automation/gallery-animation-lab.test.ts --out target/animation-lab
```

An automation test can stop application time at an exact checkpoint:

```ts
await ui.getById('increment').click()
await ui.getById('increment').click()
await ui.getById('increment').click()
await ui.pause()
await ui.screenshot('after-three-clicks.png')
await ui.resume()
```

`ui.pause()` freezes native animations, scroll physics, and JavaScript timers.
Screenshots and text or performance assertions still work. `ui.resume()`
continues from the frozen animation time. `ui.sleep(ms)` spends wall time with
the application frozen, and preserves an existing pause. `ui.wait(ms)` instead
advances the app and its timers. Both durations are bounded by the test timeout;
`ARGUI_TEST_TIMEOUT_MS` can raise that timeout to at most 300000 ms.

Windowless tests address their one mounted window as `ui.window('main')`:

```ts
const window = ui.window('main')
await window.resize({ width: 1024, height: 720, scale: 1 })
await ui.screenshot('resized.png')
```

Each resize recomputes layout and renders at the requested size; its action and
frame appear in the same performance report. The gallery resize scenario is
`tests/automation/gallery-window-resize.test.ts`. `window.move(x, y)` reports
an unsupported operation in windowless tests because there is no OS window to
position. In a native TSX application, `ApplicationServices.setWindowPosition`
requests an absolute outer position by stable window key on Windows, macOS,
and X11. `getWindowInfo` returns `x` and `y` when the OS reports them. The
native service reports unsupported on Wayland and other backends.
Native resize requests use `ApplicationServices.setWindowSize`.

The output directory contains PNG captures and `report.json`. `steps` and
`metrics.events` relate actions to their nested work; `frames` and
`frameDiagnostics` show layout, paint, render submission, and GPU passes when
the adapter supports timestamps. Resize frames also include `resizeEvents` and
`surfaceCpuMs`; the generic trace includes `render.surface_resize`.
`processMetrics` records sampled CPU and RSS.
The report names its adapter: a CPU or software adapter is useful for comparing
headless runs on the same machine, but its GPU times do not estimate physical
GPU frame time. Keep the viewport, actions, feature set, and adapter the same
when comparing two reports.
