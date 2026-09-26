# Argui TSX gallery

The gallery has widget pages plus `Examples / Layouting`, `Examples / Animation`,
`Examples / Expressive UI`, and `Examples / Scrollbars`. Expressive UI shows
five retained gradient and throbber recipes. The Scrolling widget page compares native scrolling
with a virtualized list. Solid is the default; React implements the same pages
and examples.

```sh
bun run dev
bun run dev:react
bun run dev:web:solid
bun run dev:web:react
```

`bun run dev` starts the native Solid gallery; `bun run dev:react` starts the
native React gallery. The `dev:web:*` commands start the browser galleries.
The Web commands regenerate the JSX contract, compile the gallery's WASM host,
then start Vite. Rust edits rebuild WASM and reload the browser; TSX edits use
Vite's normal reload. A WASM rebuild resets application state.
An already installed matching `wasm-bindgen` is reused without downloading it.
Open the local URL printed by Vite; the React page is `/react.html`. Both Web
pages mount the same TSX scenes and Rust renderer as the native gallery. The
current browser renderer requires WebGPU and reports a startup error when the
browser does not provide a WebGPU adapter.

The shared widget package exports `Button`, `ButtonGroup`, `Checkbox`,
`Switch`, `InputField`, `Select`, `Tabs`, `Slider`, `Progress`, `Popover`, and
`VirtualList` from `@argui/widgets/solid` or `@argui/widgets/react`.
Public props use the same camelCase names in both frameworks.
The [ButtonGroup guide](../../docs/ui/button-group.md) shows joined actions,
search, vertical orientation, and RTL composition.
The [Popover guide](../../docs/ui/popover.md) defines the controlled state,
width, focus, and overlay theme contract.
The [controls guide](../../docs/ui/controls.md) covers the five new controls.
The [theme guide](../../docs/ui/theme.md) lists the official Neutral tokens,
System mode, and accepted hex, Oklch, RGB, and RGBA values.

```tsx
<column gap={12} padding={16}>
  <text>Hello</text>
  <Button variant="default" onClick={save}>Save</Button>
  <InputField label="Name" value={name} onValueChange={setName} />
</column>
```

Create one theme runtime when mounting the app and provide it once at the root.
Widgets read the inherited theme; `ThemeScope` can override selected tokens for
a subtree.

```tsx
const runtime = createThemeRuntime(bridge, widgetThemeDefinition)
root.render(
  <ThemeProvider runtime={runtime}>
    <App />
  </ThemeProvider>,
)
```

`InputField` and `Select` support `value`, `defaultValue`, and `onValueChange`.
`Select` and `Popover` also support `open`, `defaultOpen`, and `onOpenChange`.
`VirtualList` requires a stable `itemKey` function; native scrolling determines
which bounded range is mounted.

Icons belong to the application. `Button` accepts JSX children, and the other
widgets expose optional `leading` or `trailing` slots. For example, a gallery
can provide its own assets without adding an icon dependency to the widget:

```tsx
<Button onClick={save}>
  <row gap={8} alignItems="center">
    <svg source={mediaAssets['gallery/settings.svg']} width={16} height={16} />
    <text color={theme.text}>Save</text>
  </row>
</Button>
<InputField label="Search" leading={
  <svg source={mediaAssets['tabler/search.svg']} width={16} height={16} />
} />
<Select label="Language" options={languages} trailing={
  <svg source={mediaAssets['tabler/chevron-down.svg']} width={16} height={16} />
} />
```

The old gallery and widget implementation is preserved as a read-only reference
under the repository root's `OLD_API/` directory. Active builds and exports do
not import that archive.
