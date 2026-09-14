# Argui documentation

Start with the [live components](https://extrabinoss.github.io/argui/components)
and [getting started guide](https://extrabinoss.github.io/argui/get-started).
These guides cover the current Rust APIs, supported behavior and platform limits.
Run commands from the repository root unless a guide says otherwise.

## Build an application

| Topic | Guides |
| --- | --- |
| State and runtime | [Models, ownership and services](runtime/models.md) · [Asynchronous tasks](runtime/tasks.md) · [Hot reload](hot-reload.md) |
| Layout and interaction | [Styling and themes](ui/styling.md) · [Focus, input and accessibility](ui/interaction.md) · [Scroll and virtualization](ui/scroll.md) |
| UI composition | [Localization](i18n.md) · [Animation](ui/animation.md) · [Actions and editing](ui/editing.md) · [Custom elements](ui/custom-elements.md) |
| Widgets | [Component catalogue](widgets/shadcn.md) · [Lists and tables](widgets/lists-tables.md) · [Overlays and placement](widgets/overlays.md) |
| Rendering | [Primitives, colors, images and SVG](rendering/primitives.md) · [GPU effects](rendering/effects.md) |

## Integrate with the platform

| Topic | Guide |
| --- | --- |
| Application identity, windows and tray | [Application setup](platform/application.md) |
| Native Android and iOS | [Mobile integration](native-mobile.md) |
| Safe areas and system UI | [Window insets](platform/window-insets.md) |
| Native and browser web content | [WebView](platform/webview.md) |
| System file selection | [File picker](platform/file-picker.md) |
| Blur behind window regions | [Desktop backdrops](platform/desktop-backdrops.md) |
| Popovers outside the window | [Native popovers](platform/native-popovers.md) |
| Signed application updates | [Updater engine and dialog](platform/updater.md) |

## Understand and contribute

- [Repository structure](repo/structure.md): every crate, platform boundary, dependency and publication order.
- [Architecture](architecture.md): crate boundaries and how the retained runtime works.
- [Performance](performance/optimizations.md): measured costs, raw data and reproducible workloads.
- [DevTools](contributing/devtools.md): inspection, live editing and profiling.
- [Code quality](contributing/code-quality.md): dependencies, source rules, tests and coverage.
- [Linux graphical testing](contributing/linux-testing.md): isolated displays and browser captures.
- [Releases](contributing/releases.md): crates.io publication, GitHub releases and branch protection.
- [Website](../website/README.md): Nuxt development and GitHub Pages deployment.
- [Roadmap](roadmap.md): planned capabilities and remaining validation.

Complete widget integrations live in the [gallery source](../crates/argui-widget-gallery/src/pages/).
