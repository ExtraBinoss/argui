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
`Button`, `InputField`, `Select`, `Popover`, `Dialog`, `palette`, and `inputText`.
Each widget has its own Solid and React source file under
`packages/widgets/src/solid/` and `packages/widgets/src/react/`. Shared types,
colors, input text extraction, and asset providers stay separate so component
installers can copy one widget and its declared dependencies.
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
`InputField` accepts `password` and adds a Show/Hide button. The native editor
masks the text until revealed and keeps both states protected from clipboard,
undo history, and accessibility value export. `selectionColor` overrides its
selection highlight; otherwise the theme accent uses the native selection's
38% opacity.

To compare native blur algorithms in the gallery, set `ARGUI_GALLERY_BLUR` to
`gaussian` or `dual` before launching. The default is `auto`:

```sh
ARGUI_GALLERY_BLUR=gaussian ./scripts/gallery-hot-reload.sh desktop solid
ARGUI_GALLERY_BLUR=dual ./scripts/gallery-hot-reload.sh desktop solid
```

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

## Desktop application services

Run `./scripts/gallery-hot-reload.sh desktop solid` on your own desktop, then
open **Examples → Services**. This gallery opts into a tray icon and hiding its
main window on close. Other Argui applications keep the normal quit behavior
unless they configure a tray and close policy. The page can change its
translated tray menu, enable or disable the tray, and register a global shortcut
to bring the gallery back. Native registration errors appear in the page status.

The page also opens a companion native window. Its initial title, logical size,
decorations, transparency, and desktop blur area come from TSX. Use **Send to
companion**, then click the companion to deliver its reply to the main gallery.
`ApplicationServices` in `@argui/host` reads live window information and changes
title, size, and decorations on either window. Transparency and backdrop are
creation options, so close and reopen the companion to change them. A requested
backdrop paints a solid color if the desktop cannot supply blur; window info
reports `backdropAvailable`, and native initialization failures emit an error event.

For an application that opts into a transparent window with a native desktop
backdrop, any TSX visual element can request that effect with
`desktop_backdrop_tint` and `desktop_backdrop_fallback`. Apply the pair to the
root element to cover the whole window, or to a child for a smaller region:

```tsx
<column width="fill" height="fill"
  desktop_backdrop_tint="#202c4599"
  desktop_backdrop_fallback="#202c45">
  <text text="Welcome" />
</column>
```

Keep the ancestors of a smaller region transparent. The separate
`backdrop_filter="blur(12px)"` property blurs content already painted inside
the Argui window; it does not request desktop compositor blur.

## Runtime theming

Open **Examples → Theming** in either the Solid or React gallery. The three
presets and the JSON editor change color, text size, padding, corner radius,
shadow, and blur while the native tree is mounted. The variable names are
defined by the application; Argui does not maintain an allowed token list.
A variable can be passed to any TSX property accepting its value type, so the
same `radius` value can style a container, a rectangle, and a text background.

The example parses a flat JSON object of string, finite number, and boolean
values. It accepts additional variable names. It validates only the variables
that this particular view uses before applying them to native properties.
The application can persist `themeJson(variables)` and later load it with
`parseThemeVariables(json)`; file reading and writing stay in application code.
The blur control drives `backdrop_filter="blur(Npx)"` on a translucent native
rectangle above colored shapes, so increasing it visibly softens the shapes.

The search widget gives its inner `textInput` `clip={true}` in TSX. This clips
glyphs, selection, and caret at the editor's own bounds after the search icon;
the outer rounded frame clips the complete field separately.

## Dialog widget

Open **Components → Dialog** to try the reusable `Dialog` widget from
`@argui/widgets/solid` or `@argui/widgets/react`. It uses a full-window native
modal portal, a blurred scrim, a centered panel, focus containment, Escape
dismissal, and a close button. Pass `open`, `onOpenChange`, `title`, `theme`,
and child content. `blur`, `radius`, `padding`, `scrimColor`, and `surfaceColor`
can be set from any application theme variables of the matching types.
