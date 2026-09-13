<p align="center">
  <a href="https://extrabinoss.github.io/argui/">
    <img src="https://raw.githubusercontent.com/ExtraBinoss/argui/main/website/public/argui-icon.png" width="88" height="88" alt="Argui">
  </a>
</p>

<h1 align="center">Argui</h1>

<p align="center">
  Fast, accessible interfaces. Written in Rust.<br>
  Native on Linux, macOS and Windows. At home on the web.
</p>

<p align="center">
  <a href="https://extrabinoss.github.io/argui/">Website</a> ·
  <a href="https://extrabinoss.github.io/argui/components">Live components</a> ·
  <a href="docs/README.md">Documentation</a> ·
  <a href="https://extrabinoss.github.io/argui/get-started">Get started</a>
</p>

<p align="center">
  <a href="https://github.com/ExtraBinoss/argui/actions/workflows/ci.yml"><img src="https://github.com/ExtraBinoss/argui/actions/workflows/ci.yml/badge.svg" alt="CI status"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-1a73e8" alt="MIT or Apache-2.0"></a>
</p>

[![The Argui widget gallery](https://raw.githubusercontent.com/ExtraBinoss/argui/main/website/public/gallery-preview.webp)](https://extrabinoss.github.io/argui/components)

## Build your interface in Rust

Argui brings a retained UI tree, GPU rendering and a growing widget collection
to one Rust API. Compose your interface, keep state in models and let the
runtime update what changed. The same rendering stack runs on desktop and
WebAssembly with WebGPU.

- **Responsive by design.** Flex and grid layout, light and dark themes, animation and virtualized lists.
- **Accessible controls.** Keyboard navigation, focus, text editing, native AccessKit integration and browser semantics.
- **GPU effects.** Custom WGSL shaders, gradients, shadows, blur and liquid glass.
- **Native integrations.** Multiple windows, file pickers, trays, WebViews and popovers that can extend beyond the window on supported backends.
- **Optional updates.** A signed update engine with a separate, reusable progress dialog.
- **Inspect as you build.** Element inspection, live styles, theme editing and profiling through optional DevTools.

Widgets and platform integrations are enabled individually. Idle interfaces do
not request animation frames. Read the [performance measurements](docs/performance/optimizations.md)
for workloads, memory figures and reproduction commands.

## Try it

Explore the [live gallery](https://extrabinoss.github.io/argui/components), or run it locally:

```sh
git clone https://github.com/ExtraBinoss/argui.git
cd argui
cargo run -p argui-widget-gallery --all-features
```

Use Rust 1.98 or newer. Linux builds with all features also need the
[native dependencies](docs/platform/webview.md#linux-build-dependencies).

## Compose a view

Add only the widgets your application uses:

```toml
[dependencies]
argui = { git = "https://github.com/ExtraBinoss/argui", features = ["widget-button"] }
```

```rust
use argui::{
    ui::Element,
    widgets::{Button, WidgetTheme},
};

fn view(theme: &WidgetTheme) -> Element {
    Element::row([
        Button::new("save", "Save changes", theme.button()).build(),
        Button::new("cancel", "Cancel", theme.ghost_button()).build(),
    ])
    .gap(10.0)
}
```

Continue with [models and state](docs/runtime/models.md), the
[widget catalogue](docs/widgets/shadcn.md) or the [complete examples](crates/argui/examples/).
The Git dependency works today; registry releases follow the [release workflow](docs/contributing/releases.md).

## Where things stand

Argui is under active development and its APIs are still evolving.
**Hot reload and internationalization are coming next.** See the
[roadmap](docs/roadmap.md) and individual platform guides for current support.

Contributions go through pull requests. Start with the
[contributor guide](docs/contributing/code-quality.md); every crate must meet
the 85% floor for lines, functions, regions and branches. Cargo builds use at
most six jobs. The [website](website/README.md) lives in `website/`.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
