# VirtualList DSL example

This standalone app uses the same [`ListView.argui`](../../crates/argui-dsl/stdlib/ui/list-view.argui)
as the widget gallery. The component owns scrolling, virtualization, the themed
scrollbar, and optional top/bottom edge shadows. This app leaves the shadows
off by default; the widget gallery enables them on its sidebar with
`edge_shadow_width`, `edge_shadow_intensity`, and `edge_shadow_color`. Set those
three properties on the `ListView` in `ui/main.argui` to try them here.
The caller supplies a keyed `for` template; only rows in the visible window
are rendered. The default DSL
data has 24 rows so `argui dev` works immediately; the release app injects
10,000 records from Rust without changing the DSL row template.

For live editing, run from the example directory:

```sh
cd app_examples/dsl-virtual-list
cargo run --manifest-path ../../Cargo.toml -p argui-cli --bin argui -- dev
```

Edit `ui/main.argui` while the window is open. The app rebuilds the row window
after model, scroll, or viewport changes. To run the AOT release version,
return to the repository root and execute:

```sh
cd ../..
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-dsl-virtual-list --release
```
