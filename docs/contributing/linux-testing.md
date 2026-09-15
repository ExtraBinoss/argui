# Linux graphical testing

Run every GUI check through `scripts/linux-hidden-display.sh`. It creates a
private Mutter/Wayland session and keeps test windows off the user's desktop.
Inspect saved captures; a blank capture fails.

Do not use `xdg-open`, a personal browser profile, or the user's Xwayland
server as a fallback.

## Prerequisites

The helper needs headless Mutter, `dbus-run-session`, `mountpoint`,
`fusermount3`, and an EGL driver. X11 scenarios also need Xwayland, Python
Xlib, and Pillow. [The CI installer](../../scripts/install-linux-ci.sh) installs
the Ubuntu packages.

```sh
./scripts/linux-hidden-display.sh sh -c \
  'test -z "${DISPLAY:-}" && test -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY"'
```

`ARGUI_TEST_MONITOR` changes the virtual monitor; the default is
`1600x1200@60`.

## Native application

```sh
cargo build -p argui-widget-gallery --all-features
./scripts/linux-hidden-display.sh \
  timeout 15s target/debug/argui-widget-gallery
```

Startup alone does not verify drawing or input. Use a scenario that saves PNGs
or inspect a Wayland capture.

For native X11, let the helper create a private rootful Xwayland server:

```sh
ARGUI_TEST_BACKEND=x11 ./scripts/linux-hidden-display.sh \
  timeout 20s target/debug/argui-widget-gallery

ARGUI_NATIVE_TESTS=1 cargo nextest run \
  -p argui-runtime --all-features --test native_popups
```

`native_popups` creates its own private display and writes captures to
`target/native-popups/`.

## Browser application

Build and serve the gallery:

```sh
wasm-pack build crates/argui-widget-gallery --target web --dev \
  --out-dir ../../web/widgets/pkg --all-features
python3 scripts/dev_server.py 8793 --directory web --entry /widgets/
```

Run a fresh Chromium process inside the private display:

```sh
CHROME_PATH=/path/to/chrome \
PUPPETEER_MODULE=/path/to/puppeteer/lib/esm/puppeteer/puppeteer.js \
GALLERY_URL=http://127.0.0.1:8793/widgets/ \
./scripts/linux-hidden-display.sh \
  node crates/argui-widget-gallery/tests/pages.mjs
```

The browser checks use visible Chromium inside the hidden compositor, Wayland,
Vulkan WebGPU, and ANGLE Vulkan. A populated accessibility tree does not prove
that the GPU canvas painted.

Common scenario entry points:

| Area | Script under `crates/argui-widget-gallery/tests/` |
| --- | --- |
| General widgets | `pages.mjs` |
| Editors and selection | `pages/inputs.mjs` |
| Overlays and effects | `pages/overlay_effects.mjs` |
| Widget catalogue | `pages/catalogue.mjs` |
| Animation and scrolling | `pages/motion.mjs` |
| Localization | `pages/i18n.mjs` |
| Virtual lists | `pages/data.mjs` |
| Application navigation | `app/navigation.mjs` |
| Desktop backdrop | `app/desktop_backdrop.mjs` |

Accessibility and DevTools have separate entry points:

```text
crates/argui-accessibility/tests/web.mjs
crates/argui-devtools/tests/view.mjs
```

Set `GALLERY_URL` explicitly and use `SCREENSHOT_DIR` when supported. Inspect
the resulting PNGs for content, contrast, clipping, focus, and frame changes.

## Native Wayland capture

`scripts/linux-wayland-capture.py` uses ScreenCast and RemoteDesktop inside the
private session:

```sh
./scripts/linux-hidden-display.sh timeout --signal=INT --kill-after=3s 55s \
  python3 scripts/linux-wayland-capture.py \
    --output target/capture \
    -- target/debug/argui-widget-gallery
```

It needs PyGObject, Pillow, GStreamer with `pipewiresrc`, PipeWire, and
WirePlumber. It rejects a non-private display and a capture without contrasting
content.

## Final gate

Run targeted scenarios while editing. After implementation, run the complete
gate once:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

Do not run another coverage process concurrently. See
[code quality](code-quality.md).
