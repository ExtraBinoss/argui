# Linux graphical testing

Run every GUI check through `scripts/linux-hidden-display.sh`. It starts a
private Mutter compositor, virtual monitor and D-Bus session, with a temporary
runtime directory accessible only to its owner. Test windows stay off your
desktop. Inspect saved captures; a blank image is a failed visual check.

The helper removes the inherited display sockets and session bus, then shuts
down its compositor and cleans up private FUSE mounts when the command exits.
A failed helper is not a reason to run tests on the normal desktop. Do not use
`xdg-open`, an existing browser profile or the user's desktop Xwayland server.

## Prerequisites

The basic helper needs Mutter with headless Wayland support, `dbus-run-session`,
`mountpoint`, `fusermount3` and an EGL driver. X11 scenarios additionally need
Xwayland, Python Xlib and Pillow. The [CI installer](../../scripts/install-linux-ci.sh)
sets up the Ubuntu dependencies. The helper has been exercised with Mutter 50.4
on the development machine and Mutter 46 on the Ubuntu runner.

```sh
# Verify isolation without opening a window.
./scripts/linux-hidden-display.sh sh -c \
  'test -z "${DISPLAY:-}" && test -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY"'

# Build with the same features used by the graphical checks.
cargo build -p argui-widget-gallery --all-features
./scripts/linux-hidden-display.sh timeout 15s target/debug/argui-widget-gallery
```

A deliberate timeout returns a nonzero status. Startup alone does not validate
rendering or interaction. `ARGUI_TEST_MONITOR` sets the virtual monitor, for
example `1920x1200@60`; the default is `1600x1200@60`.

## Native X11

`ARGUI_TEST_BACKEND=x11` starts a rootful Xwayland server inside the private
Wayland compositor. The helper picks a free display, disables TCP and removes
the Wayland socket from the application's environment. It terminates only the
Xwayland process it started. Do not call `linux-hidden-x11.sh` directly.

```sh
ARGUI_TEST_BACKEND=x11 ./scripts/linux-hidden-display.sh \
  timeout 20s target/debug/argui-widget-gallery

# This opt-in test creates its own private compositor and X server.
ARGUI_NATIVE_TESTS=1 cargo nextest run -p argui-runtime --all-features \
  --test native_popups
```

Input drivers verify the private server's PID and environment before using
XTest. The `native_popups` scenario checks surfaces outside the parent window,
`WM_TRANSIENT_FOR` ownership, editing, selection shortcuts, clicks, scrolling
and focus loss. Captures are saved in `target/native-popups/`. Black space
around the test windows is the private server's unused framebuffer.

## Browser checks

Build and serve the gallery, then run a fresh Chromium instance in the private
display. Reuse an installed Puppeteer module through `PUPPETEER_MODULE`; never
connect to a personal browser session.

```sh
wasm-pack build crates/argui-widget-gallery --target web --dev \
  --out-dir ../../web/widgets/pkg --all-features
python3 scripts/dev_server.py 8793 --directory web --entry /widgets/
```

In another terminal:

```sh
CHROME_PATH=/path/to/chrome \
PUPPETEER_MODULE=/path/to/puppeteer/lib/esm/puppeteer/puppeteer.js \
GALLERY_URL=http://127.0.0.1:8793/widgets/ \
./scripts/linux-hidden-display.sh \
  node crates/argui-widget-gallery/tests/pages.mjs
```

The checked browser configuration uses `headless: false` inside the invisible
compositor, Wayland, Vulkan WebGPU and `--use-angle=vulkan`. SwiftShader produced
blank captures on the development machine. A populated accessibility tree does
not establish that GPU content was painted.

The website has its own [browser scenario](../../website/README.md#verify).
Gallery scripts accept `GALLERY_URL`; defaults vary by scenario, so set it
explicitly when using a different server. Most accept `SCREENSHOT_DIR` as well.

## Scenario reference

Replace the final script in the command above with the scenario you need.
Paths below are relative to `crates/argui-widget-gallery/tests/` unless noted.

| Area | Script | What it exercises |
| --- | --- | --- |
| Common controls | `pages.mjs` | Labels, breadcrumbs, pagination, skeletons, collapsibles, menus, calendar and data tables. |
| Editors and menus | `pages/inputs.mjs` | Shortcuts, word selection, dragging, nested menus and stable field bounds. |
| Overlays | `pages/overlay_effects.mjs` | Popovers, tooltips, effects, focus, dismissal and native-surface fallback. |
| Catalogue | `pages/catalogue.mjs` | Forms, navigation, modal surfaces, Hover Card and scrolling in both themes. |
| Color picker | `pages/color_picker.mjs` | Immediate preview colors, alpha input, stable geometry and GPU color checks. |
| Animated text | `pages/animated_text.mjs` | Changed digits, carries, fades, intermediate frames and narrow layouts. |
| Updates | `pages/updater.mjs` | Notes, download amounts, unknown totals, cancellation, installation states and errors. |
| Liquid glass | `pages/liquid_glass.mjs` | Refraction, controls and content scrolling behind the glass. |
| Scroll effects | `pages/scroll_effects.mjs` | Vertical and horizontal edges and light-theme contrast. |
| Virtual lists | `pages/data.mjs` | Wheel replay, visible rows, CPU profiles and frame intervals. |
| Navigation | `app/navigation.mjs` | Search, first-key input, keyboard navigation and retained editor shortcuts. |
| Desktop backdrop | `app/desktop_backdrop.mjs` | Transparency controls, fallback and canvas alpha. |
| Accessibility | `crates/argui-accessibility/tests/web.mjs` from the repository root | DOM stability, focus, virtual row replacement, order and accessible actions. |
| DevTools | `crates/argui-devtools/tests/view.mjs` from the repository root | Inspection, panes, themes, color editing and resource controls. |

Inspect the saved PNGs for text, contrast, spacing, clipping and disabled states.
Argui may retain DOM focus on the canvas: check `aria-activedescendant` and the
resulting input behavior as well as `document.activeElement`. Native screen-reader
checks remain separate from browser DOM assertions.

For native DevTools and a detached tools window:

```sh
cargo build -p argui-widget-gallery --release --all-features
ARGUI_TEST_BACKEND=x11 ./scripts/linux-hidden-display.sh \
  python3 crates/argui-devtools/tests/app.py
```

`--sensors` enables an installed all-smi executable. `--seconds 10` adds CPU/RSS
sampling; do not compile or run other tests while measuring. `--binary`,
`--output` and `--baseline` support paired comparisons. See
[performance](../performance/optimizations.md) for measurement limits.

## Native Wayland captures

This helper uses ScreenCast/RemoteDesktop services on the **private Mutter
session** and starts its own PipeWire instance. It requires Python/PyGObject,
Pillow, GStreamer with `pipewiresrc`, PipeWire and WirePlumber.

```sh
./scripts/linux-hidden-display.sh timeout --signal=INT --kill-after=3s 55s \
  python3 scripts/linux-wayland-capture.py \
  --output target/desktop-backdrop/wayland \
  --click settings:1110:260 --click glass:884:385 \
  --click transparency:884:664 --click sidebar:1110:260 \
  -- target/debug/argui-widget-gallery
```

The private WirePlumber policy opens no audio, Bluetooth or camera devices.
The helper rejects a non-private display and a capture without contrasting
content. Inspect `initial.png` before reusing click coordinates; they assume
the default 1600 × 1200 virtual monitor. `--settle` defaults to 25 seconds to
allow portals to start. No permission or capture applies to the user's session.

A window that stopped presenting is not a valid animation or scrolling benchmark,
even if it reports very low CPU usage. Verify visible frame changes throughout
performance measurements.

## Final quality gate

Follow [code quality](code-quality.md): targeted checks during development, no
concurrent LLVM coverage runs, then one complete gate before committing:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

This includes the opt-in native tests and keeps the 85% threshold for each
coverage metric, globally and in every crate.
