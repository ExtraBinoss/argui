# Argui — Another Rust GUI

Build fast, accessible interfaces in Rust. Argui combines a retained UI tree,
GPU rendering, modular widgets and native platform integrations across Linux,
macOS, Windows and the web. The same renderer runs natively and through
WebAssembly/WebGPU.

[Website and live components](https://extrabinoss.github.io/argui/) ·
[Documentation](docs/README.md) ·
[Contributing](docs/contributing/code-quality.md) ·
[Releases](docs/contributing/releases.md)

Argui is experimental. Its core builds on `winit`, `wgpu`, `cosmic-text` and
`taffy`, with an optional WebView and no required JavaScript frontend.

The repository provides a retained UI tree, responsive layout, shaped text and
editing, interaction, scrolling, animation, transforms, gradients, decoded
images, scoped GPU effects, and one WGPU renderer for native and web. See the
[visual primitive contracts](docs/rendering/primitives.md), the
[color contract](docs/rendering/primitives.md#color), and the
[overlay geometry API](docs/widgets/overlays.md#overlay-geometry), or the
[remaining roadmap](docs/roadmap.md).

Performance work includes retained subtree reuse, virtualization, compact tree
indices, dense layout storage, early dirty-propagation stops, and explicit
layout boundaries for independently sized scroll content.
See [the optimization measurements](docs/performance/optimizations.md) for reproducible
before/after CPU and memory comparisons, workloads, and their limits.
The [quality gate](docs/contributing/code-quality.md) requires at least 85% coverage on each
metric, both across the workspace and within every crate.

```sh
cargo nextest run --workspace --all-features
./scripts/quality.sh
cargo run -p argui --example window
cargo run -p argui --example text
cargo run -p argui --example layout
cargo run -p argui --example state
cargo run -p argui --example spotlight
cargo run -p argui-perf-showcase --example perf
./scripts/serve-web.sh
```

The native `state` example and the web build launch the same `StateShowcase`
Rust application from `argui-showcase`; only their tiny platform entrypoints
differ. Both wrap it in the optional `DevtoolsHost`, whose resizable Elements
and Profiling dock is itself composed from normal Argui widgets. Applications
that do not want inspection simply launch their `Render` component directly. Its fonts are
embedded because browsers do not expose system font files
to WASM. Argui itself ships no mandatory font; each application supplies the
assets and generic-family mapping it wants.

The `layout` example uses the retained `UiTree` API. Taffy computes logical
rectangles from the current viewport and Cosmic Text supplies intrinsic text
measurement, so native-window and browser resizing share the same reflow path.

The state showcase embeds a transparent PNG and a JPEG. `argui-image` is an
optional decoding boundary; the renderer receives validated RGBA assets and
does not depend on a file format or filesystem.

Browse the [documentation index](docs/README.md) for API guides, platform
integration, performance evidence and contributor instructions.

## Optional asynchronous tasks

Enable `argui/updater` for the UI-independent application update engine and
`argui/widget-updater` separately for its optional dialog. The gallery's `updater`
feature demonstrates version notes, download progress, cancellation and installation
states. See [application updates](docs/platform/updater.md) for signed feeds and
the supported desktop package formats.

Enable `argui/tasks` for owned, cancellable asynchronous tasks independently
of WebView support: Tokio on native and browser futures on Web.
See [the task contract](docs/runtime/tasks.md) and **Examples → Async tasks** in the gallery.

## Actions and editing

Scoped actions share commands across buttons, keyboard shortcuts, menus and
palettes. Text editors retain bounded transactional undo/redo; Password fields
mask graphemes and suppress clipboard export/history. Menu and command palette
widgets are individually opt-in. See [the API guide](docs/ui/editing.md) and
the **Actions** and **Editing & Password** gallery pages.
Reusable data widgets are documented in [Lists and tables](docs/widgets/lists-tables.md);
[the shadcn catalogue](docs/widgets/shadcn.md) maps all 64 entries to public APIs,
feature flags, examples, and their supported scope.

Enable `argui/desktop-backdrop` for native desktop blur through selected UI
regions, with configurable tint and fallback. See [Desktop backdrops](docs/platform/desktop-backdrops.md)
and the gallery's **Appearance** controls.

## Optional WebView

`argui-webview` provides retained sessions, a bounded native-view cache and a
Wry backend on desktop and a sandboxed iframe backend on the web. Enable the
`webview` feature on `argui` for its re-export.
The runtime mounts native content from the retained layout. WebView-enabled
Linux applications use the GTK/Tao Wayland host; ordinary applications retain
Winit. See [the integration status and security model](docs/platform/webview.md).

`./scripts/serve-widget-gallery.sh` enables WebView support automatically.
Open **Examples → WebView → Email** at `/widgets/` to render sanitized email
HTML in the browser. Webpage mode can embed only sites that permit iframes.
The script also starts the separate-origin relay for the Webpage **isolated /
compatible** toggle. Its origin allowlist and typed popup/download permissions
are documented in [WebView configuration](docs/platform/webview.md#explicit-webpage-permissions).

Linux build prerequisites and host limitations are in
[WebView setup](docs/platform/webview.md#linux-build-dependencies).
Native file selection and its optional widget are documented in
[File picker](docs/platform/file-picker.md).

## Website

The [Nuxt website](website/README.md) presents the library, features and component
catalogue in English. Its Components page embeds the real WebAssembly gallery
and links each component to its Rust implementation.

```sh
cd website
pnpm install --frozen-lockfile
pnpm gallery:build
pnpm dev
```
