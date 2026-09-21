# Widget Gallery DSL

This is the independent, hot-reloadable gallery for Argui DSL. The existing
Rust gallery remains available for comparison. The shell and each page live in
separate `.argui` files. `Button`, `Input`, `Card`, `Badge`, `Separator`, and
`Switch`, and `VirtualList` are reusable modules in
`crates/argui-dsl/stdlib/ui/`; Media exercises native image and SVG primitives.
The content area has no enclosing card. The navigation list and the
[standalone large-data example](../dsl-virtual-list/) use the same
`VirtualList.argui` component with different caller-authored row templates.

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
The Input page restores the earlier controlled, search, invalid, read-only and
disabled examples, plus caret and real text-selection playgrounds. Select the
ordinary sentences to compare a theme-colored highlight with the rounded
3-pixel default and an authored conic gradient; select text in the inputs to
compare linear, radial, and conic GPU highlights. Focus the caret fields to
compare the animated bar, dot, and three-dot gradient. The separate Virtual
List page uses the same standard component as the sidebar and the standalone
large-model example.

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
- [ ] The same theme tokens respond to light/dark/accent selection and custom
  theme mode overrides.
- [ ] All component pages, keyboard interaction, and resize behavior pass tests
  and private-display visual inspection.
- [ ] Dev and warning-free release builds pass, followed by the final quality
  gate.
