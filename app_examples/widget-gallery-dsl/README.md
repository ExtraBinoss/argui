# Widget Gallery DSL

This is the independent, hot-reloadable gallery for Argui DSL. The existing
Rust gallery remains available for comparison. The shell and each page live in
separate `.argui` files. The Linux gallery includes a DSL page for every one
of the 79 default Rust gallery entries, plus DSL-specific composition and
scrollbar examples. Reusable widgets live in `crates/argui-dsl/stdlib/ui/` and
are composed from native rectangles, text, focus scopes, touch areas, and
scroll viewports. Media exercises native image and SVG primitives. The
sidebar contains Components and Examples in one scrolling list. The VList
page and the [standalone large-data example](../dsl-virtual-list/) share the
same `ListView.argui` component with caller-authored rows.

From this example directory, start the interactive development build:

```sh
cargo run --manifest-path ../../Cargo.toml -p argui-cli --bin argui -- dev
```

Edit `ui/main.argui`, one of `ui/pages/*.argui`, or a standard-library module in
`../../crates/argui-dsl/stdlib/ui/`. `argui dev` reports the source change,
compilation result and client acknowledgement. The window should update without
an extra click. Stop with Ctrl+C.

For a generated release build, run from the repository root:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-widget-gallery-dsl --release
```

The UI is responsive: the sidebar occupies the full workspace height and
scrolls independently with a themed scrollbar and optional scroll-reactive
top/bottom edge shadows; at narrow widths the content wraps below it. Appearance
is controlled by a select-style button and popover. The same `--argui-*` token
names resolve to light, dark, violet or emerald palettes. A custom theme can
override the same tokens in its corresponding mode. The Media page imports
local PNG, WebP and SVG assets; Button imports Tabler icons through `@argui/icons`. Only
referenced icon assets are bundled in a generated build.

The Button page counts every enabled click and reports the last variant used.
The Menu page builds a desktop menu bar with nested Export and Transform
flyouts. Its status line reports the clicked leaf action. Menu triggers open on
click and switch on hover while a menu is active. `SubMenu` takes a caller-owned
`open` value and `activate`, `hover`, and `dismiss` callbacks, so the caller can
coordinate sibling flyouts. Give each submenu a named anchor container.
The **Examples** category contains Scrollbar styling. It shows a quiet
thumb, a wide track, a colored hover state, and a thumb with a WGSL effect
composed from `Flickable`, `TouchArea`, and `Rectangle`. Both `ScrollView` and
`ListView` reserve a rail beside the scrollable content.
The Overlays page includes a translucent Popover with adjustable
`surface_fill`, `border_color`, `radius`, and `backdrop_filter`. Its second popup
is assembled directly from `PopupWindow` and `Rectangle`; it runs
[`worley-border.wgsl`](assets/worley-border.wgsl) only on the rectangle border.
This demonstrates how to author a different popup surface without changing the
renderer or adding a native widget.
The Popover page includes a striped backdrop and an interactive filter selector
for each CSS-style filter, Argui's refraction and color-matrix filters, and an
ordered combination. Its filter list is authored through the same
`backdrop_filter` property available on base visual primitives.
The Input page restores the earlier controlled, search, invalid, read-only and
disabled examples, plus caret and real text-selection playgrounds. Select the
ordinary sentences to compare a theme-colored highlight with the rounded
3-pixel default and an authored conic gradient; select text in the inputs to
compare linear, radial, and conic GPU highlights. Focus the caret fields to
compare the animated bar, dot, and three-dot gradient. The separate VList
page uses the same standard component as the sidebar and the standalone
large-model example.

The File picker page calls the platform's native system dialog through a
root callback. It supports single and multiple files, folders, and a save
destination in both AOT and live development. On desktop, the callback waits
for the dialog result before the gallery can respond to other input; cancellation
and platform errors are shown in the page. The browser build reports that these
system dialogs require the native gallery. Other older Rust examples still
depend on host services or native controls unavailable to these DSL pages:
embedded WebView, cancellable background tasks, renderer damage telemetry,
and OS update or mobile activity APIs. Those pages describe their limits
instead of reporting fabricated results. The old gallery's mobile and
updater pages are conditional on platform or feature and are not part of
the 79 default pages.

## Acceptance checklist

- [x] Full-height, themed sidebar scroll and narrow-layout navigation work.
- [x] The topbar and borderless content use all available width.
- [x] Each enabled button variant updates a shared counter and last-used label.
- [x] Input restores controlled, search, invalid, read-only and disabled fields;
  caret and text-selection gradients are authored in DSL.
- [x] The dedicated Virtual List page scrolls the same reusable DSL component.
- [ ] Button loading spinner animates as an SVG, including after live updates.
- [ ] Arbitrary project image and SVG assets render in AOT and live mode; asset
  edits hot-reload and invalid imports report a source diagnostic.
- [ ] Tabler imports are feature-gated, report unknown icons, and bundle only
  referenced SVGs in release builds.
- [x] The same theme tokens respond to light/dark/accent selection and custom
  theme mode overrides.
- [ ] All component pages, keyboard interaction, and resize behavior pass tests
  and private-display visual inspection.
- [ ] Dev and warning-free release builds pass, followed by the final quality
  gate.
