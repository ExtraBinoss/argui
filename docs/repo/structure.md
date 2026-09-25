# Repository structure

The repository separates the Rust UI engine from framework adapters and
applications. Rust applications depend on the engine and runtime crates they
need; TSX applications use the host and framework packages under `packages/`.

## Top-level directories

| Path | Contents |
| --- | --- |
| `assets/fonts/` | Fonts and licenses shared by engine tests and applications |
| `apps/gallery/` | Solid and React TSX gallery, asset host, and QuickJS runner |
| `crates/` | Rust engine, platform, and integration libraries |
| `mobile/android/` | Gradle `NativeActivity` shell for the TSX gallery |
| `packages/` | TypeScript host, Solid and React adapters, and shared widgets |
| `web/` | Static browser helper scripts |
| `docs/` | Architecture, contracts, platform limits, and contributor guides |
| `scripts/` | Quality, profiling, packaging, release, and development tools |
| `tests/scripts/` | Tests for repository automation |

## Dependency direction

```text
Rust applications                         TSX applications
      |                                          |
argui-runtime                            @argui/widgets
      |                                  @argui/react / @argui/solid
      |                                          |
      |                                      @argui/host
      +-------------------+----------------------+
                          |
              argui-host + argui-schema
                          |
       ui + layout + renderer + platform
                          |
   core + paint + text + animation + accessibility
```

Dependencies point toward the engine. Lower Rust layers do not import
application or framework code. `argui-runtime` composes native Rust
applications; the TSX adapters submit host transactions through
`argui-host` and the shared schema. Both paths use the same retained UI and
rendering engine.

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
| `argui-media` | Stable media assets and optional image/SVG decoding | core, paint |
| `argui-platform` | Winit windows, input, clipboard, tray, and OS adapters | core, paint |
| `argui-text` | Shaping, bidi, fallback, cursor geometry, and glyphs | core |
| `argui-render` | WGPU surfaces, resources, batching, effects, and WGSL validation | core, paint, text |
| `argui-theme` | Theme tokens and appearance settings | core |
| `argui-ui` | Retained elements, styles, events, focus, and semantics | core, accessibility, animation, paint, text |
| `argui-layout` | Retained Taffy Flexbox and Grid adapter | core, paint, text, ui |
| `argui-effects` | Optional WGSL effect definitions | paint, render, ui |
| `argui-i18n` | Optional Fluent catalogs and locale negotiation | — |
| `argui-updater` | Signed update checks, downloads, and installation | — |
| `argui-webview` | Retained native WebViews and browser frames | core, layout, ui |
| `argui-runtime` | Models, scheduling, windows, layout, and rendering | engine and platform crates |
| `argui-host` | Native transaction host used by the TSX adapters | schema, ui, runtime |
| `argui-schema` | Shared element and transaction contract | core |

Publishable Rust engine crates use the root workspace release policy. The
gallery's application-only Rust crates have independent manifests and are not
part of the root Cargo workspace.

## Target selection

| Target | Integration |
| --- | --- |
| Linux, Windows, macOS | Rust applications use `argui-runtime` and the needed engine crates |
| WebAssembly | Rust `cdylib` with a `#[wasm_bindgen(start)]` launcher |
| Android | `argui-runtime/mobile/android` feature; the repository gallery uses its QuickJS host and TSX bundles |
| iOS | `argui-runtime/mobile/ios` feature; applications provide their own native shell |

Desktop and Web implementations are selected by Cargo target configuration.
Android and iOS entry features are separate and target-gated. This
repository currently packages the Android gallery; it no longer contains an
iOS sample application or Xcode project. Packaging details are in
[native mobile](../native-mobile.md).

## Choosing a location

- Put shared value types in `argui-core` only when several lower layers need
  them and they carry no platform or GPU dependency.
- Put immutable drawing descriptions in `argui-paint`; put resource upload and
  shader execution in `argui-render`.
- Put retained interaction behavior in `argui-ui`; put framework-neutral TSX
  widget logic in `packages/widgets/src/shared` and Solid/React views in the
  matching `solid` and `react` directories.
- Put JavaScript host integration in `packages/host` and framework-specific
  rendering adapters in `packages/solid` or `packages/react`.
- Put Winit and OS APIs in `argui-platform`; route application-visible results
  through `argui-runtime`.
- Put runnable TSX demonstrations in `apps/gallery`; keep engine behavior in
  its owning Rust crate.

Adding a cross-layer type is a design decision. Prefer translating it at the
boundary over making a lower crate depend on a higher one.

## Publication

`python3 scripts/release.py package` reads Cargo metadata, validates public
manifests, computes the dependency order, and compiles every archive. The CI
publisher uses the same order, skips versions already owned by the project, and
publishes crates in dependency order. Do not maintain a second hard-coded order
in docs.

See [releases](../contributing/releases.md) for the commit trigger and recovery
procedure.
