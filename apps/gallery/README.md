# Native Solid and React gallery

`scripts/build-gallery.sh` regenerates the Rust schema contract, the media
manifest, and both framework bundles, then builds the native QuickJS runner.
The Solid application is the default; the same native contract and media IDs
are used by the React bundle. Bun is only a build and test tool, not an app
runtime.

## Media imports

Place SVG, PNG, JPEG, or WebP files under `apps/gallery/assets/`, then run
`bun run build:gallery`. This creates one manifest and matching TypeScript and
Rust imports. Use a generated reference in either adapter, for example
`<svg source={mediaAssets['tabler/heart.svg']} />` or
`<image source={mediaAssets['photo/saturn.jpg']} />`. The native runner embeds
and decodes those files once, and registers their stable IDs with the renderer.

For another Tabler icon, copy its raw SVG from the
[Tabler Icons repository](https://github.com/tabler/tabler-icons) into
`apps/gallery/assets/tabler/`, then rebuild. The gallery uses SVG files
directly; framework icon wrappers target a DOM renderer and are not needed.
The bundled Tabler sources are MIT licensed; see `assets/README.md`.

Raster images and SVGs render natively. Video is not exposed as a component in
this slice. Argui has image/vector assets and texture rendering, but no native
video decoder, playback clock, audio synchronization, frame upload lifecycle,
or video semantics. GStreamer supplies cross platform SDKs and `appsink` for
decoded frames, so it is a possible backend after those pieces and mobile
packaging are integrated. The gallery deliberately has no video substitute.

## Same-scene Animation Lab comparison

The comparison runner starts on Animation Lab with loops playing. `quickjs` keeps
Solid/QuickJS and native host transactions alive. `rust` runs the same generated
Solid bundle once, opens Animation Lab, commits its operations into `Host`, and
clones the resulting validated `Element` into an immutable direct Rust
`AppModel`. QuickJS is dropped before the Rust window starts. Both variants use
the same renderer settings, native loop implementation, assets, safe-area wrapper,
and platform viewport. The Rust snapshot's controls cannot call back into
QuickJS, but native scrolling and loops remain active.

Build the bundle and a feature-free desktop binary once, then run either
variant on the private display. `scripts/build-gallery.sh` also builds this
feature-free binary:

```sh
bun run build:gallery
cargo build --manifest-path apps/gallery/quickjs-host/Cargo.toml
./scripts/linux-hidden-display.sh env ARGUI_GALLERY_COMPARE=quickjs \
  timeout 30s cargo run --manifest-path apps/gallery/quickjs-host/Cargo.toml --locked
./scripts/linux-hidden-display.sh env ARGUI_GALLERY_COMPARE=rust \
  timeout 30s cargo run --manifest-path apps/gallery/quickjs-host/Cargo.toml --locked
```

For a graphical check, use `scripts/linux-wayland-capture.py` inside
`linux-hidden-display.sh` and inspect its saved PNG. A blank capture fails.
Run the same viewport and scroll gesture for each presentation.

Android Activity does not inherit `adb shell` environment variables, so build
one variant at a time with the Cargo feature selected by Gradle. Both commands
use the same debug build profile and the Pixel's actual viewport:

```sh
./scripts/android-gallery.sh install -ParguiGallery=solid -ParguiAbis=arm64-v8a -ParguiComparison=quickjs
adb logcat -c
./scripts/android-gallery.sh launch -ParguiGallery=solid
adb logcat | rg 'argui-comparison'
# Stop the app, then rebuild and repeat with -ParguiComparison=rust.
./scripts/android-gallery.sh install -ParguiGallery=solid -ParguiAbis=arm64-v8a -ParguiComparison=rust
```

The default Android and desktop gallery remains on Button unless comparison
mode is selected. `--all-features` test builds select the Rust variant when both
comparison features are enabled; production comparison builds select one.

The runner prints a fixed-size 120-frame sample without logging per frame:
frame interval p95, model/tree/paint and render mean times, and damaged pixels
as a fraction of viewport pixels. The live variant also prints host batch and
operation counts after startup, JavaScript callback work, and native host queue
plus commit time. It reports click callback latency and time from a large host
commit to the next render profile. That last measurement excludes the host
commit itself and is not a display-present timestamp. Native render and
animation profiles have different scopes and must not be added together.

To verify that both presentations receive the same generated scene and that
native loops do not cause JavaScript commits:

```sh
cargo nextest run --manifest-path apps/gallery/quickjs-host/Cargo.toml \
  --all-features -E 'binary(comparison)' --run-ignored all
```
