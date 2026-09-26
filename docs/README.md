# Repository documentation

These guides explain repository boundaries, engine contracts, platform limits,
testing, and releases. The website will be rebuilt for the TSX API separately.

Start with:

1. [Architecture](architecture.md) for the frame and event flow.
2. [Repository structure](repo/structure.md) to find the crate that owns a change.
3. [Solid and React native gallery](solid-react-native.md) for the shared host, selected QuickJS runtime, and measured limits.
4. [Development guide](contributing/development.md) and [code quality](contributing/code-quality.md) before opening a pull request.

For applications using the previous TSX API, see [the v2 migration guide](migration-tsx-v2.md).

## Engine and application contracts

| Area | Guide |
| --- | --- |
| Models and lifetime | [Models](runtime/models.md) · [Tasks](runtime/tasks.md) |
| Layout and input | [Layout](ui/layout.md) · [Styling](ui/styling.md) · [Interaction](ui/interaction.md) · [Scroll](ui/scroll.md) |
| UI behavior | [Editing](ui/editing.md) · [Animation](ui/animation.md) · [Custom elements](ui/custom-elements.md) |
| Rendering | [Primitives](rendering/primitives.md) · [Text fidelity](rendering/text.md) · [Adaptive damage](rendering/damage.md) · [Compositor](rendering/compositor.md) · [Effects](rendering/effects.md) · [GPU canvases](rendering/gpu-canvas.md) |
| Localization | [Internationalization](i18n.md) |
| TSX widgets and assets | [Gallery and framework adapters](../apps/gallery/README.md) · [Application icons](ui/icons.md) |

## Platform integration

| Area | Guide |
| --- | --- |
| Windows and application setup | [Application](platform/application.md) |
| Android and iOS | [Native mobile](native-mobile.md) · [Safe areas](platform/window-insets.md) |
| Native services | [File picker](platform/file-picker.md) · [WebView](platform/webview.md) · [Updater](platform/updater.md) |
| Desktop surfaces | [Backdrops](platform/desktop-backdrops.md) · [Native popovers](platform/native-popovers.md) |

## Maintainer guides

- [Performance](performance/optimizations.md): contracts and measurements.
- [Native automation](automation.md): deterministic captures and performance reports.
- [Linux graphical testing](contributing/linux-testing.md): private displays and
  capture-based checks.
- [Releases](contributing/releases.md): packaging, crates.io, and GitHub automation.
- [Roadmap](roadmap.md): open work only.

Commands in these guides run from the repository root unless stated otherwise.
