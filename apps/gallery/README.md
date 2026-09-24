# Native Solid and React gallery

`scripts/build-gallery.sh` regenerates the Rust schema contract, the media
manifest, and both framework bundles, then builds the native QuickJS runner.
The Solid application is the default; the same native contract and media IDs
are used by the React bundle. Bun is only a build and test tool, not an app
runtime.

## Shared TSX widgets

The gallery's reusable controls live in the workspace package
[`@argui/widgets`](../../packages/widgets/package.json). Import them from
`@argui/widgets/solid` or `@argui/widgets/react`; both entries expose
`Button`, `InputField`, `Select`, `Popover`, `palette`, and `inputText`.
Applications provide their own native asset references through
`WidgetAssetProvider`, so the package does not depend on the gallery's generated
asset manifest:

```tsx
import { Button, palette, WidgetAssetProvider } from '@argui/widgets/solid'
import { mediaAssets } from './assets.generated'

const theme = palette('dark', 'blue')

function App() {
  return (
    <WidgetAssetProvider icons={{ search: mediaAssets['tabler/search.svg'] }}>
      <Button id="save" label="Save" theme={theme} kind="primary" onClick={() => {}} />
    </WidgetAssetProvider>
  )
}
```

For React, import the same names from `@argui/widgets/react`.

## Native internationalization

The gallery imports its English and French Fluent catalogs from
`apps/gallery/src/i18n/` in both Solid and React bundles. The
[`@argui/i18n` package](../../packages/i18n/README.md) exposes
`loadI18n`, `tr`, and `selectLocale`; the framework entries provide a reactive
`useI18n()` hook. JSON catalog edits are Vite dependencies, so hot reload builds
and reloads the updated catalog. Release builds embed the same generated bundle.
Rust parses and formats each Fluent message through the QuickJS bridge.

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

The TSX gallery (`argui-gallery-quickjs`) can include all 5,944 Tabler SVG
definitions in a **development bundle**. Enable the matching native feature
once, then TSX changes and icon selection use the existing hot reload without
another native build:

```sh
ARGUI_GALLERY_ALL_TABLER=1 ./scripts/gallery-hot-reload.sh desktop solid
# Android: install one debug APK with -ParguiDevTabler=true first.
./scripts/android-gallery.sh install -ParguiAbis=arm64-v8a -ParguiDevTabler=true
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
3 MB of source before Vite minification. The source is generated from the MIT
licensed `icondata_tb` 0.1.0 package (and Tabler Icons, see
`assets/tabler/LICENSE`). Refresh it explicitly with
`python3 apps/gallery/scripts/generate-dev-tabler.py`; use `--check` to verify
the pinned 5,944-icon output. Normal builds never run this generator.

Raster images and SVGs render natively. Video is not exposed as a component in
this slice. Argui has image/vector assets and texture rendering, but no native
video decoder, playback clock, audio synchronization, frame upload lifecycle,
or video semantics. GStreamer supplies cross platform SDKs and `appsink` for
decoded frames, so it is a possible backend after those pieces and mobile
packaging are integrated. The gallery deliberately has no video substitute.

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
./scripts/android-gallery.sh install -ParguiAbis=arm64-v8a
./scripts/gallery-hot-reload.sh android solid
# Replace solid with react to push the React bundle to the same debug app.
```

The Android script uses `run-as` to atomically update the app's private
`files/gallery-core.mjs` and requires an attached device with USB debugging.
Release builds always use the embedded Solid bundle. For a custom desktop
build loop, set `ARGUI_GALLERY_BUNDLE` to an absolute path to a Solid or React
bundle and run `vite build --watch` separately.
