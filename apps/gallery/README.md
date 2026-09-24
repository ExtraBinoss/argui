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

### Optional complete Tabler catalogue during development

The TSX gallery (`argui-gallery-quickjs`, separate from the Rust
`argui-widget-gallery`) can include all 5,944 Tabler SVG definitions in a
**development bundle**. Enable the matching native feature once, then TSX changes and icon
selection use the existing hot reload without another native build:

```sh
ARGUI_GALLERY_ALL_TABLER=1 ./scripts/gallery-hot-reload.sh desktop solid
# Android: install one debug APK with -ParguiDevTabler=true first.
./scripts/android-gallery.sh install -ParguiGallery=solid -ParguiAbis=arm64-v8a -ParguiDevTabler=true
ARGUI_GALLERY_ALL_TABLER=1 ./scripts/gallery-hot-reload.sh android solid
```

Import `tablerIcon` and `tablerIconNames` from `./tabler-icons` in either Solid or
React TSX. For example, `<svg source={tablerIcon('arrow-up')} color={theme.accent} />`.
The first use sends only that SVG to the native renderer; the other icons remain
unparsed. Changing the TSX selection requires only a bundle reload. The default
bundle omits the complete catalogue and continues to include the small checked-in
SVG set. The native registration control is available only in debug builds with
`dev-tabler-icons`; release builds reject it and the release bundle omits the
catalogue. Keep the catalogue on the local development build; it adds roughly
3 MB of source before Vite minification. The source is generated from the MIT licensed `icondata_tb` 0.1.0 package
(and Tabler Icons, see `assets/tabler/LICENSE`). Refresh it explicitly with
`python3 apps/gallery/scripts/generate-dev-tabler.py`; use `--check` to verify
the pinned 5,944-icon output. Normal builds never run this generator.

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

## Native virtual lists

Both adapters export `VirtualList` from `@argui/solid` and `@argui/react`.
The component needs only an item count and a render function:

```tsx
<VirtualList count={items.length} renderItem={(index) =>
  <text text={items[index].label} />
} />
```

The native scroll/layout engine measures each mounted item, corrects the scroll
anchor, and requests a new bounded range only when the visible chunk changes.
The TSX component mounts that range; it does not run JavaScript for every scroll
pixel. `axis="horizontal"` measures widths, while the default vertical axis
measures heights. Optional `estimate`, `overscan`, `itemKey`, `width`, `height`,
`scrollbarVisible`, `scrollbarWidth`, `scrollbarColor`, and `shadow` control the
presentation. `shadow` accepts theme color, intensity, feather width, and
individual left/top/right/bottom edges. The gallery uses a horizontal list on
mobile and a vertical sidebar on desktop.

The count is finite and can grow by appending items. Variable extents retain
memory proportional to the count (about 9 MiB for one million items). If items
are inserted, removed, or reordered in the middle, change `dataVersion` to
reset index-based measurements; appending or truncating the tail preserves the
measured prefix.

## Native TSX hot reload

Install or build the native runner once. Subsequent TSX edits only rebuild a
Vite bundle; the running QuickJS host reloads it without rebuilding Rust or
restarting the window. The previous scene stays mounted when the new JavaScript
or native transaction fails. A successful reload remounts the gallery, so local
component state and the selected page return to their initial values.

```sh
# Desktop, inside the private display required for GUI checks:
./scripts/linux-hidden-display.sh ./scripts/gallery-hot-reload.sh desktop solid
# Replace solid with react for the React gallery.

# Android: requires an attached device and the debug Solid APK installed once.
# Reinstall only when Rust/native code changes; then keep this running:
./scripts/android-gallery.sh install -ParguiGallery=solid -ParguiAbis=arm64-v8a
./scripts/gallery-hot-reload.sh android solid
# Replace solid with react to push the React bundle to the same debug app.
```

The Android script uses `run-as` to atomically update the app's private
`files/gallery-core.mjs` and requires an attached device with USB debugging.
Release builds always use the embedded Solid bundle. For a custom desktop
build loop, set `ARGUI_GALLERY_BUNDLE` to an absolute path to a Solid or React
bundle and run `vite build --watch` separately.
