<p align="center">
  <a href="https://extrabinoss.github.io/argui/">
    <img src="https://raw.githubusercontent.com/ExtraBinoss/argui/main/website/public/argui-icon.png" width="88" height="88" alt="Argui">
  </a>
</p>

<h1 align="center">Argui</h1>

<p align="center">
  Fast, accessible interfaces authored in <code>.argui</code>.<br>
  Native on Linux, macOS and Windows; WebAssembly and mobile-ready.
</p>

<p align="center">
  <a href="https://extrabinoss.github.io/argui/">Website</a> ·
  <a href="https://extrabinoss.github.io/argui/components">Components</a> ·
  <a href="docs/dsl/getting-started.md">Get started</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="https://discord.gg/xY9CWSc65">Discord</a>
</p>

<p align="center">
  <a href="https://github.com/ExtraBinoss/argui/actions/workflows/ci.yml"><img src="https://github.com/ExtraBinoss/argui/actions/workflows/ci.yml/badge.svg" alt="CI status"></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-1a73e8" alt="MIT or Apache-2.0"></a>
</p>

[![The Argui widget gallery](https://raw.githubusercontent.com/ExtraBinoss/argui/main/website/public/gallery-preview.webp)](https://extrabinoss.github.io/argui/components)

## Start with the DSL

Argui’s primary UI API is a typed declarative language. Rust remains the
application and engine language, while ordinary layout, controls, bindings,
states, themes, animation declarations and event routing live in `.argui` files.

From this repository, create and run a project with:

```sh
cargo run -p argui-cli -- new hello-argui
cd hello-argui
cargo run
```

The generated application owns its UI in `ui/main.argui`:

```text
import { Button, Column, Text } from "@argui/ui"

export component Main {
    private property expanded: bool = true
    callback toggled(expanded: bool)

    Column {
        gap: 12.0
        Text { text: "Welcome to Argui" }
        Button {
            text: expanded ? "Hide details" : "Show details"
            on click { expanded = !expanded; toggled(expanded) }
        }
        if expanded {
            Text { text: "The UI is authored in .argui" }
        }
    }
}
```

`build.rs` invokes `argui-dsl-build`; Cargo then compiles generated, typed Rust.
Normal release dependencies contain neither parser, compiler, interpreter,
watcher, live protocol nor LSP.

## Develop with transactional reload

Inside a generated application, run:

```sh
argui dev
```

The command starts the application once, watches `.argui`, `.wgsl` and imported
assets, and publishes versioned packages over TCP and WebSocket. Valid changes
are prepared and committed atomically. Invalid source or WGSL produces
diagnostics while the previous UI remains visible. Compatible component state,
focus/editing identity, keyed repeater instances, animation slots and asset
handles survive a reload; a public Rust-facing ABI change asks for a restart.

Useful non-interactive commands are:

```sh
argui check ui/main.argui --json
argui fmt --check
argui schema Button --json
argui complete ui/main.argui 12:9 --json
argui symbols --json
argui-dsl-lsp
```

See the [DSL development guide](docs/dsl/development.md) for native, browser and
mobile clients, compatibility boundaries, and editor setup.

## Connect business logic

Exported components generate a strongly typed Rust API. Input/output properties,
callbacks and slots are the deliberate boundary between application logic and
the UI. No string lookup occurs on the release render path.

```rust
argui::include_ui!();

let root = Main::new();
root.on_toggled(|expanded| {
    // Application service or domain logic.
});
```

For Fluent localization, bind the existing `argui-i18n` localizer to DSL
`tr("message.id")` expressions:

```rust
let localizer = std::rc::Rc::new(localizer);
root.set_translator({
    let localizer = std::rc::Rc::clone(&localizer);
    move |id| localizer.text(id).ok()
});
```

## What the language provides

- Multi-file modules with explicit imports and exports.
- Typed structs, enums, models, properties, callbacks and slots.
- One-way and explicit two-way bindings with cycle diagnostics.
- Keyed repeaters and source-stable retained identity.
- Typed themes, derived tokens, runtime modes and named styles.
- Conditional states and retained animation declarations.
- External WGSL effects validated against Argui’s ABI.
- Reachable image, vector and shader assets with stable live revisions.
- Fluent-compatible `tr()` expressions and accessible native behaviors.
- One canonical typed IR shared by AOT and live execution.
- Formatter, LSP and stable JSON commands for editors and automation.

The [language guide](docs/dsl/language.md) documents the supported surface, and
`argui schema --json` is the canonical machine-readable component contract.

## Engine capabilities

The same retained engine underneath the DSL supplies:

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

The DSL’s official application-facing library is `@argui/ui`. Rust element and
widget APIs remain available for engine extensions, native behavior adapters
and specialized rendering—not as the default application authoring path.

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
