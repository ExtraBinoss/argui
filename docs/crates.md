# Crates, dependencies, and features

For a Rust application in Argui 0.4, start with `argui-runtime`. The
[`argui init rust`](cli.md#create-an-application) scaffold creates a
`Cargo.toml` pinned to its own CLI version and imports its public runtime
types. There is no single `argui` umbrella crate in this workspace. A Solid
or React TSX application should use the CLI-generated packages and host;
it does not need to list engine crates manually.

## Choose the dependency you need

| Need | Direct dependency | When to add it |
| --- | --- | --- |
| A native Rust application | `argui-runtime` | Start here; it composes windows, models, input, layout, and rendering. |
| Native file selection or platform APIs | `argui-platform` | Use its API directly; enable `file-picker` for dialogs. |
| Fluent translations and locale negotiation | `argui-i18n` | Add when the app has translated messages. |
| Signed application updates | `argui-updater` | Add only when shipping an updater. |
| Custom WGSL effects | `argui-effects` | Add for effect definitions used by the renderer. |
| Lower-level engine integration | `argui-ui`, `argui-layout`, `argui-paint`, `argui-render`, `argui-text` | Add only if the app actually imports those APIs. |
| Transaction host or custom adapter | `argui-host`, `argui-schema` | For host and adapter authors; ordinary apps use the runtime or CLI. |

`argui-core`, `argui-animation`, `argui-theme`, `argui-accessibility`,
`argui-media`, and `argui-inspect` are focused layers. Add one directly only
when using its public types. Cargo resolves the runtime's internal
dependencies for you; copying the whole workspace into an app's manifest is
unnecessary. The [repository structure](repo/structure.md#crates) guide lists
every crate and its dependency direction.

## Add runtime features

The generated Rust manifest is equivalent to:

```toml
[dependencies]
argui-runtime = "=0.4.0"
```

Add only the capabilities the app uses:

```toml
[dependencies]
argui-runtime = { version = "=0.4.0", features = ["tasks", "tray"] }
argui-platform = { version = "=0.4.0", features = ["file-picker"] }
argui-i18n = "=0.4.0"
```

These Cargo commands add the same exact version requirements:

```sh
cargo add argui-runtime@=0.4.0 --features tasks,tray
cargo add argui-platform@=0.4.0 --features file-picker
cargo add argui-i18n@=0.4.0
argui check
```

`cargo add` changes dependencies; `argui check` validates the application.
For a generated Rust app, `argui init rust --feature tasks` also enables
runtime tasks at creation. For TSX apps, `argui init solid --feature tasks`
wires the matching native host feature. The CLI's `--feature` choices at init
are `tasks` and, for native TSX, `automation`; this is separate from the
full Cargo feature set below.

| `argui-runtime` feature | Adds | Typical use |
| --- | --- | --- |
| `tasks` | Async task scheduling | Background work attached to app models. |
| `inspect` | `argui-inspect` records | Custom inspection or tooling. |
| `media` | Host media transaction support | Media-backed host content. |
| `tray` | `argui-platform/tray` | System tray integration. |
| `global-shortcuts` | `argui-platform/global-shortcuts` | OS-level key bindings. |
| `native-popups` | `argui-platform/native-popups` | Popups beyond the app window where supported. |
| `desktop-backdrop` | `argui-platform/desktop-backdrop` | Acrylic, Mica, or native backdrop effects. |
| `window-input-regions` | `argui-platform/window-input-regions` | Pointer holes and custom hit regions. |
| `webview` | WebView and its native host dependencies | Embedded Web content on supported desktops. |
| `android`, `ios` | Runtime entry hooks | Native mobile shells supplied by the app. |

The runtime defaults to no optional features. Desktop and Web rendering
backends are chosen by the Cargo target, not by a `web` or `desktop` feature.
Platform support is still target-specific: for example, the WebView path
brings GTK dependencies on Linux, and native popup behavior depends on the
window backend. Read [desktop windows](platform/desktop.md) and
[native mobile](native-mobile.md) before enabling those integrations.

Some capabilities live in their own crates rather than a runtime feature:

| Capability | Dependency or feature |
| --- | --- |
| File picker | `argui-platform` with `file-picker`. |
| Localization | `argui-i18n`. |
| Signed updater | `argui-updater`. |
| WGSL effects | `argui-effects` with only its required effect features. |
| Native test driver | The CLI's `--feature automation` for generated native TSX apps, or `argui-automation` for a custom harness. |

`argui-effects` offers `blur`, `shadow`, `color`, `refraction`, `liquid-glass`,
`artistic`, and `scroll`. For example, an app using only blur can add
`argui-effects = { version = "=0.4.0", features = ["blur"] }`. Avoid enabling
all effects just to use one shader.

To see the actual resolved graph in a Rust project, use
`cargo tree -e features` or `cargo tree -e features -i argui-runtime`. These are Cargo
dependency diagnostics; use `argui check` for the Argui application itself.
The root [Cargo manifest](../Cargo.toml) owns the current versions and the
minimum Rust version for source builds.
