# Repository documentation

The [documentation website](https://extrabinoss.github.io/argui/docs) teaches
the public API with runnable examples. The files in this directory explain the
repository: ownership boundaries, implementation contracts, platform limits,
testing, and releases.

Start with:

1. [The 0.3 application API](simplified-api.md) for the public learning path.
2. [Architecture](architecture.md) for the frame and event flow.
3. [Repository structure](repo/structure.md) to find the crate that owns a change.
4. [Development guide](contributing/development.md) for the edit and test loop.
5. [Code quality](contributing/code-quality.md) before opening a pull request.

## Engine and application contracts

| Area | Guide |
| --- | --- |
| Models and lifetime | [Models](runtime/models.md) · [Tasks](runtime/tasks.md) |
| Layout and input | [Styling](ui/styling.md) · [Interaction](ui/interaction.md) · [Scroll](ui/scroll.md) |
| UI behavior | [Editing](ui/editing.md) · [Animation](ui/animation.md) · [Custom elements](ui/custom-elements.md) |
| Rendering | [Primitives](rendering/primitives.md) · [Compositor](rendering/compositor.md) · [Effects](rendering/effects.md) · [GPU canvases](rendering/gpu-canvas.md) |
| Optional capabilities | [Localization](i18n.md) · [Hot reload](hot-reload.md) |
| Widgets | [Interaction APIs](widgets/interaction-api.md) · [Builder inventory](widgets/builder-inventory.md) · [Catalogue](widgets/shadcn.md) · [Lists and tables](widgets/lists-tables.md) · [Overlays](widgets/overlays.md) |

## Platform integration

| Area | Guide |
| --- | --- |
| Windows and application setup | [Application](platform/application.md) |
| Android and iOS | [Native mobile](native-mobile.md) · [Safe areas](platform/window-insets.md) |
| Native services | [File picker](platform/file-picker.md) · [WebView](platform/webview.md) · [Updater](platform/updater.md) |
| Desktop surfaces | [Backdrops](platform/desktop-backdrops.md) · [Native popovers](platform/native-popovers.md) |

## Maintainer guides

- [Performance](performance/optimizations.md): contracts, measurements, and
  reproducible profiling commands. The [feature profile measurements](performance/feature-profiles.md)
  cover the facade's convenience aliases.
- [DevTools](contributing/devtools.md): inspection and profiling behavior.
- [Linux graphical testing](contributing/linux-testing.md): private displays and
  capture-based checks.
- [Releases](contributing/releases.md): packaging, crates.io, and GitHub automation.
- [Website](../website/README.md): local development and GitHub Pages.
- [Roadmap](roadmap.md): open work only.

Commands in these guides run from the repository root unless stated otherwise.
