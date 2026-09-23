<p align="center">
  <a href="https://extrabinoss.github.io/argui/">
    <img src="https://raw.githubusercontent.com/ExtraBinoss/argui/main/website/public/argui-icon.png" width="88" height="88" alt="Argui">
  </a>
</p>

<h1 align="center">Argui</h1>

<p align="center">
  Fast, accessible native interfaces with a retained Rust renderer.<br>
  Native on Linux, macOS and Windows; WebAssembly and mobile-ready.
</p>

<p align="center">
  <a href="https://extrabinoss.github.io/argui/">Website</a> ·
  <a href="https://extrabinoss.github.io/argui/components">Components</a> ·
  <a href="docs/README.md">Get started</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="https://discord.gg/xY9CWSc65">Discord</a>
</p>

<p align="center">
  <a href="https://github.com/ExtraBinoss/argui/actions/workflows/ci.yml"><img src="https://github.com/ExtraBinoss/argui/actions/workflows/ci.yml/badge.svg" alt="CI status"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-1a73e8" alt="MIT or Apache-2.0"></a>
</p>

[![The Argui widget gallery](https://raw.githubusercontent.com/ExtraBinoss/argui/main/website/public/gallery-preview.webp)](https://extrabinoss.github.io/argui/components)

## Native UI engine

Argui provides native layout, text, input, animation, accessibility and WGPU
rendering. `argui-schema` describes the native primitives; `argui-ui::UiTree`
retains their identity and interaction state. The [Solid and React native gallery](docs/solid-react-native.md)
uses one shared host and the embedded QuickJS runtime on this engine.

## Engine capabilities

The retained engine supplies:

- flex/grid layout, scrolling, virtualization and responsive presentation;
- WGPU rendering on desktop, Android, iOS and WebAssembly;
- text shaping, editing, selection, bidirectional text and IME input;
- mouse, touch, keyboard, focus and AccessKit/browser semantics;
- typed animation, custom WGSL, gradients, shadows, blur and liquid glass;
- retained WGPU canvases and adaptive damage rendering;
- optional DevTools, file picker, updater, WebView, tray, global shortcuts,
  native popovers and desktop backdrop.

Idle applications do not request animation frames. Reproducible measurements
are in the [performance guide](docs/performance/optimizations.md).

## Platforms and optional capabilities

Linux, Windows, macOS and WebAssembly are Cargo targets rather than feature
flags. Android and iOS use the opt-in `argui-android` and `argui-ios` entry
crates. The facade’s optional integrations include `i18n`, `tasks`, `devtools`,
`file-picker`, `updater`, `webview`, `tray`, `global-shortcuts`,
`native-popups`, `desktop-backdrop`, `widgets-all`, and individual `widget-*`
features for advanced Rust engine integrations.

Read the [native mobile guide](docs/native-mobile.md), [repository structure](docs/repo/structure.md),
and [platform guides](docs/README.md) for integration details.

## Explore and contribute

Run the complete gallery locally:

```sh
git clone https://github.com/ExtraBinoss/argui.git
cd argui
cargo run -p argui-widget-gallery --release
```

Contributions follow the [development guide](docs/contributing/development.md)
and [quality gate](docs/contributing/code-quality.md). Every crate must meet the
85% floor for lines, functions, regions and branches.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
