# Repository structure

The workspace separates engine layers, optional integrations, and applications.
Application code normally depends on `argui`; engine code depends on the
narrowest crate that owns the required type.

## Top-level directories

| Path | Contents |
| --- | --- |
| `crates/` | Rust libraries, launchers, tests, and native examples |
| `app_examples/` | Standalone applications used by the website |
| `mobile/android/` | Gradle `NativeActivity` shell for the Widget Gallery |
| `mobile/ios/` | Xcode shell and XCFramework integration |
| `web/` | Generated and static WebAssembly hosts |
| `website/` | Nuxt documentation site |
| `docs/` | Architecture, contracts, platform limits, and contributor guides |
| `scripts/` | Quality, profiling, packaging, release, and local-server tools |
| `tests/scripts/` | Tests for repository automation |

## Dependency direction

```text
applications
    |
argui facade + optional integrations
    |
runtime + widgets
    |
ui + layout + renderer
    |
core + paint + text + animation + accessibility
```

Dependencies point downward. Lower layers do not import application code,
widgets, or the facade. `argui-runtime` is the composition root; `argui`
selects features and re-exports public APIs.

## Crates

The “Depends on” column lists workspace dependencies, omitting optional entries
when the purpose already names the integration.

| Crate | Purpose | Depends on |
| --- | --- | --- |
| `argui-core` | Geometry, color, input, and shared identifiers | — |
| `argui-accessibility` | Semantic tree plus native/browser adapters | core |
| `argui-animation` | Timelines, interpolation, springs, and decay | core |
| `argui-inspect` | Renderer-independent inspection records | core |
| `argui-paint` | Ordered renderer-independent display list | core |
| `argui-image` | Image decoding and paint resources | paint |
| `argui-platform` | Winit windows, input, clipboard, tray, and OS adapters | core, paint |
| `argui-text` | Shaping, bidi, fallback, cursor geometry, and glyphs | core |
| `argui-render` | WGPU surfaces, resources, batching, and effects | core, paint, text |
| `argui-theme` | Theme tokens and appearance settings | core |
| `argui-ui` | Retained elements, styles, events, focus, and semantics | core, accessibility, animation, paint, text |
| `argui-layout` | Retained Taffy Flexbox and Grid adapter | core, paint, text, ui |
| `argui-effects` | Optional WGSL effect definitions | paint, render, ui |
| `argui-vector` | SVG parsing and vector paint resources | core, paint |
| `argui-i18n` | Optional Fluent catalogs and locale negotiation | — |
| `argui-updater` | Signed update checks, downloads, and installation | — |
| `argui-webview` | Retained native WebViews and browser frames | core, layout, ui |
| `argui-runtime` | Models, scheduling, windows, layout, and rendering | engine and platform crates |
| `argui-android` | Android `NativeActivity` entry point | platform, render, runtime, text |
| `argui-ios` | iOS static-library entry point | runtime |
| `argui-testing` | Renderer-independent application harness | accessibility, layout, runtime, text, ui |
| `argui-widgets` | Individually feature-gated controlled widgets | runtime, theme, ui, supporting engines |
| `argui-devtools` | Inspector, style editing, profiling, and tools UI | runtime, inspect, render, widgets |
| `argui` | Public facade and feature routing | public engine crates |
| `argui-widget-gallery` | Component and platform integration gallery | argui plus optional integrations |
| `argui-showcase` | Shared showcase model used by native and web launchers | engine and widget crates |
| `argui-web-demo` | WebAssembly launcher for the showcase | argui, devtools, showcase |
| `argui-state-app` | Packaged native state showcase | argui, devtools, showcase |
| `argui-perf-showcase` | Focused native and WebAssembly workloads | argui |

The first 24 crates in the table through `argui` are published. Applications
and showcases set `publish = false`.

## Target selection

| Target | Integration |
| --- | --- |
| Linux, Windows, macOS | `argui::runtime::run_application` |
| WebAssembly | a `cdylib` with a `#[wasm_bindgen(start)]` launcher |
| Android | explicit `argui-android` dependency and `android_main!` |
| iOS | explicit `argui-ios` dependency and `ios_main!` |

Desktop and Web implementations are selected by Cargo target configuration.
Android and iOS remain separate opt-in dependencies, so `argui --all-features`
does not add mobile entry crates. Packaging details are in
[native mobile](../native-mobile.md).

## Choosing a location

- Put shared value types in `argui-core` only when several lower layers need
  them and they carry no platform or GPU dependency.
- Put immutable drawing descriptions in `argui-paint`; put resource upload and
  shader execution in `argui-render`.
- Put retained interaction behavior in `argui-ui`; compose reusable product
  controls in `argui-widgets`.
- Put Winit and OS APIs in `argui-platform`; route application-visible results
  through `argui-runtime`.
- Put runnable demonstrations in an application crate, never in an engine crate.

Adding a cross-layer type is a design decision. Prefer translating it at the
boundary over making a lower crate depend on a higher one.

## Publication

`python3 scripts/release.py package` reads Cargo metadata, validates public
manifests, computes the dependency order, and compiles every archive. The CI
publisher uses the same order, skips versions already owned by the project, and
publishes `argui` last. Do not maintain a second hard-coded order in docs.

See [releases](../contributing/releases.md) for the commit trigger and recovery
procedure.
