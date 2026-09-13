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
- **Keep state while editing.** Optional Subsecond hot patching refreshes Rust views and handlers in native debug builds.
- **Fluent i18n is here.** Optional locale negotiation, fallback chains, variables, plurals and
  live locale switching are provided by `argui-i18n` and demonstrated in the Widget Gallery.

Widgets and platform integrations are enabled individually. Idle interfaces do
not request animation frames. Read the [performance measurements](docs/performance/optimizations.md)
for workloads, memory figures and reproduction commands.

## Feature status

- [x] Retained application models, scoped state, tasks and multi-window commands
- [x] Flexbox, grid, scrolling, virtualization and responsive layout
- [x] WGPU rendering on native desktop and WebAssembly
- [x] Text shaping, editing, selection, bidirectional text and IME input
- [x] Mouse, touch, keyboard, focus and accessible semantics
- [x] Light/dark themes, animation, images, SVG and custom WGSL effects
- [x] Fluent localization through the optional `i18n` feature
- [x] State-preserving Subsecond patches through the optional `hot-reload` feature
- [x] Optional DevTools, file picker, updater, WebView, tray, native popovers and desktop backdrop
- [x] Android and iOS bootstrap crates plus compile-checked Widget Gallery entry points

The public integration flags are `android`, `ios`, `i18n`, `hot-reload`, `tasks`,
`devtools`, `devtools-all-smi`, `file-picker`, `updater`, `widget-updater`,
`webview`, `tray`, `native-popups`, `desktop-backdrop` and `widgets-all`.
`argui-effects` additionally exposes `artistic`, `blur`, `color`, `liquid-glass`,
`refraction`, `scroll` and `shadow`. Every flag is opt-in; `argui` has no default
feature bundle.

### Choose only what the application uses

| Need | Feature(s) | Targets |
| --- | --- | --- |
| Core runtime, WGPU renderer, layout and text | none | Linux, Windows, macOS, WebAssembly |
| Android native entry point | `android` | Android; required for the facade launcher |
| iOS native entry point | `ios` | iOS; required for the facade launcher |
| Fluent locale negotiation, messages and plurals | `i18n` | All targets |
| Async model tasks | `tasks` | All targets |
| Every widget | `widgets-all` | All targets |
| A small widget set | the matching `widget-*` flags | All targets |
| Inspector and profiler | `devtools` | Desktop and WebAssembly |
| NVIDIA/AMD/Intel sensor collection | `devtools-all-smi` | Supported desktop hosts |
| State-preserving Rust patches | `hot-reload` | Native desktop debug builds |
| System file dialogs | `file-picker`; add `widget-file-picker` for its UI | Desktop and browser |
| Signed native update engine and dialog | `updater`, `widget-updater` | Desktop |
| Retained WebViews | `webview` | Desktop and browser frame support |
| System tray | `tray` | Linux, Windows, macOS |
| Windows outside the main surface | `native-popups` | Linux, Windows, macOS |
| Acrylic, Mica and native blur materials | `desktop-backdrop` | Linux, Windows, macOS |

Linux, Windows, macOS and WebAssembly are selected by the Cargo target and do
not need an OS feature. A focused application can enable capabilities directly:

```toml
[dependencies]
argui = { git = "https://github.com/ExtraBinoss/argui", default-features = false, features = [
  "i18n", "tasks", "widget-button", "widget-input",
] }
```

For mobile, add the native entry feature to the same dependency:

```toml
# Android
argui = { git = "https://github.com/ExtraBinoss/argui", features = ["android", "i18n", "widgets-all"] }

# iOS
argui = { git = "https://github.com/ExtraBinoss/argui", features = ["ios", "i18n", "widgets-all"] }
```

Available widget flags are `widget-accordion`, `widget-alert`,
`widget-alert-dialog`, `widget-animated-text`, `widget-aspect-ratio`,
`widget-attachment`, `widget-avatar`, `widget-badge`, `widget-breadcrumb`,
`widget-bubble`, `widget-button`, `widget-button-group`, `widget-calendar`,
`widget-card`, `widget-carousel`, `widget-chart`, `widget-checkbox`,
`widget-collapsible`, `widget-color-picker`, `widget-combobox`,
`widget-command-palette`, `widget-context-menu`, `widget-data-table`,
`widget-date-picker`, `widget-dialog`, `widget-direction`, `widget-drawer`,
`widget-empty`, `widget-field`, `widget-file-picker`, `widget-hover-card`,
`widget-icons`, `widget-input`, `widget-input-group`, `widget-input-otp`,
`widget-item`, `widget-kbd`, `widget-label`, `widget-list`, `widget-marker`,
`widget-menu`, `widget-menubar`, `widget-message`, `widget-message-scroller`,
`widget-native-select`, `widget-navigation-menu`, `widget-pagination`,
`widget-popover`, `widget-progress`, `widget-questionnaire`,
`widget-radio-group`, `widget-range`, `widget-scroll-area`, `widget-select`,
`widget-separator`, `widget-sheet`, `widget-sidebar`, `widget-skeleton`,
`widget-slider`, `widget-spinner`, `widget-split-pane`, `widget-switch`,
`widget-table`, `widget-tabs`, `widget-text-selection`, `widget-textarea`,
`widget-toast`, `widget-toggle`, `widget-toggle-group`, `widget-tooltip`,
`widget-tree-view`, `widget-typography` and `widget-vlist`.

### Platform roadmap

- [x] Linux native application and CI coverage
- [x] Windows native implementation and complete-workspace CI compile
- [x] macOS native implementation and complete-workspace CI compile
- [x] WebAssembly application and browser gallery
- [ ] Android release support — `argui-android` and its native gallery entry now
  compile; emulator/device validation, mobile services and packaging remain
- [ ] iOS release support — `argui-ios` and its native gallery entry now compile;
  Xcode simulator/device validation, mobile services and signing remain

Follow the [native mobile integration guide](docs/native-mobile.md) for the
current Android/iOS architecture, build commands and completion checklist. The
[repository structure](docs/repo/structure.md) lists every crate, direct
dependency and crates.io publication position.

## But… what about package size?

Features stay opt-in so applications only compile the integrations they choose.
The table below measures the real Linux x86-64 Widget Gallery executable with
Rust 1.98.0. “Base gallery” already contains every widget, Fluent i18n, tasks,
effects and DevTools; “all features” additionally enables the updater, WebView,
native popups, desktop backdrop and all-smi support.

| Profile | Gallery features | Executable | After `strip` |
| --- | --- | ---: | ---: |
| Debug | Base gallery (`--no-default-features`) | 98.63 MiB | 29.73 MiB |
| Debug | `hot-reload` | 100.29 MiB | 30.40 MiB |
| Debug | `--all-features` | 107.00 MiB | 33.23 MiB |
| Release | Base gallery (`--no-default-features`) | 36.04 MiB | 26.81 MiB |
| Release | `hot-reload` | 36.03 MiB | 26.79 MiB |
| Release | `--all-features` | 39.21 MiB | 29.17 MiB |

The development bridge costs 1.66 MiB in the debug executable, or 0.67 MiB
after stripping. In release, the 0.02 MiB difference is code-generation noise:
Argui compiles out its Subsecond connection and dispatch path, and the linker
does not retain the unused patch engine. Enabling the feature can still increase
release compilation time because Cargo builds its optional dependencies.

Reproduce a row with `cargo build -p argui-widget-gallery --bin
argui-widget-gallery --no-default-features`, adding `--features hot-reload` or
`--all-features`, and `--release` for the release rows. Sizes are `stat -c %s`
converted with 1 MiB = 1,048,576 bytes; stripped values come from a copied
binary processed by GNU `strip`. This measures the executable itself and
excludes shared system libraries and installer compression.

## Try it

Explore the [live gallery](https://extrabinoss.github.io/argui/components), or run it locally:

```sh
git clone https://github.com/ExtraBinoss/argui.git
cd argui
cargo run -p argui-widget-gallery --all-features
```

For state-preserving Rust patches, install Dioxus CLI and run the gallery with
`dx serve --package argui-widget-gallery --features hot-reload --hot-patch`.
The [hot-reload guide](docs/hot-reload.md) explains the executable layout and
the changes that still require a restart.

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

Continue with [models and state](docs/runtime/models.md),
[localization](docs/i18n.md), [hot reload](docs/hot-reload.md), the
[widget catalogue](docs/widgets/shadcn.md) or the [complete examples](crates/argui/examples/).
The Git dependency works today. A final `[PUBLISH]` commit on `main` lets the
protected CI publish the current version in dependency order; later releases
also bump that version. See the [release workflow](docs/contributing/releases.md).

## Where things stand

Argui is under active development and its APIs are still evolving.
**Fluent internationalization and native hot reload are available as optional
features.** See the
[roadmap](docs/roadmap.md) and individual platform guides for current support.

Contributions go through pull requests. Start with the
[contributor guide](docs/contributing/code-quality.md); every crate must meet
the 85% floor for lines, functions, regions and branches. Cargo builds use at
most six jobs. The [website](website/README.md) lives in `website/`.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
