# Argui gallery

The gallery mounts the same Rust element tree through Solid and React. Each
adapter has 12 widget pages: Button, ButtonGroup, Checkbox, InputField,
Popover, Progress, Select, Slider, Switch, Tabs, Tooltip, and VirtualList.
The Examples section demonstrates layout, scrollbars, animation, expressive
motion, and the desktop Screen spotlight. `ScrollShadow` is also exported by
the widget package and used for the gallery navigation.

## Run it

From the repository root:

```sh
bun install --frozen-lockfile
bun run dev
bun run dev:react
bun run dev:web:solid
bun run dev:web:react
```

The first two commands start the native QuickJS gallery with Solid or React.
The Web commands regenerate the JSX contract, compile the WASM host, and start
Vite; the React page is `/react.html`. Rust edits rebuild WASM and reload the
browser, resetting gallery state. Browser rendering needs WebGPU. On Linux,
run graphical checks through
[the private-display helper](../../docs/contributing/linux-testing.md) and
inspect a saved capture.

## Code and capabilities

`src/solid/gallery.tsx` and `src/react/gallery.tsx` own the page navigation;
their page modules are the runnable examples. The controls come from
`@argui/widgets/solid` and `@argui/widgets/react`; the current export and state
contracts are documented in [UI and widgets](../../docs/ui/README.md).
Assets are generated from `assets.config.json` by
`scripts/generate-assets.mjs`. The selected QuickJS host lives under
`quickjs-host/`; the browser bridge is in `src/web-mount.ts` and the WASM host
under `web-host/`.

The Screen spotlight requires an X11 desktop backend with compositing and
shaped input regions. It is not a browser or Wayland capability. Native
outside-window popups also depend on the runtime's `native-popups` feature;
the browser host keeps popup content inside its canvas. See
[desktop windows](../../docs/platform/desktop.md) for the supported backend
matrix and coordinate rules.

## Native TSX hot reload

For the selected adapter, the helper builds an initial bundle, watches TSX
changes with Vite, and delivers successful rebuilds to the running QuickJS
host. Run a Linux desktop window inside the private display:

```sh
./scripts/linux-hidden-display.sh ./scripts/gallery-hot-reload.sh desktop solid
```

Use `react` for the React adapter. For an installed Android debug gallery,
`./scripts/gallery-hot-reload.sh android solid` pushes changed bundles through
`adb`; the script also accepts `react` for development. Android release
packages embed Solid. See [Android packaging](../../mobile/android/README.md)
for the build and installation steps.

Run `bun run check:ts` for generated types and `bun run test:ts` for both
adapter and host interactions. A successful typecheck alone does not prove
that a transaction mounts, and a headless host test does not establish visual
quality; use a private-display capture for graphical changes.
