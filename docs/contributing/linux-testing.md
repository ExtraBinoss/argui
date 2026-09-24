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

## Native TSX gallery

The gallery uses Solid or React TSX, a Rust transaction host, and embedded
QuickJS. Install the JS workspace dependencies once with `bun install`. Run
the native gallery and keep it open while Vite watches for bundle changes:

```sh
./scripts/linux-hidden-display.sh ./scripts/gallery-hot-reload.sh desktop solid
# Use react for the React adapter.
```

The window runs until interrupted. The watcher rebuilds only the selected TSX
bundle; successful updates are delivered to the running host. See the
[gallery guide](../../apps/gallery/README.md#native-tsx-hot-reload) for the
reload behavior and Android workflow.

Run static and behavior checks without opening a window:

```sh
bun run check:ts
bun run test:ts
```

## Runtime scenarios

The native popup regression test creates its own private display and writes
captures to `target/native-popups/`:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 \
  cargo nextest run -p argui-runtime --all-features --test native_popups
```

For renderer/runtime investigation, capture the live QuickJS gallery through
Wayland:

```sh
./scripts/linux-hidden-display.sh timeout --signal=INT --kill-after=3s 55s \
  python3 scripts/linux-wayland-capture.py \
    --output target/capture \
    -- cargo run --manifest-path apps/gallery/quickjs-host/Cargo.toml --locked
```

`scripts/linux-wayland-capture.py` uses ScreenCast and RemoteDesktop inside the
private session. It needs PyGObject, Pillow, GStreamer with `pipewiresrc`,
PipeWire, and WirePlumber. It rejects a non-private display and a capture
without contrasting content.

The TSX gallery's shared host and framework behavior is exercised by
`bun run test:ts`; it is not a browser application.

## Final gate

Run targeted scenarios while editing. After implementation, run the complete
gate once:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

Do not run another coverage process concurrently. See
[code quality](code-quality.md).
