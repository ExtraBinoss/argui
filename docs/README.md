# Argui documentation

Argui is a retained Rust UI engine with Solid and React TSX adapters. The
desktop TSX host embeds QuickJS; the Web host runs the same Rust renderer in
WebAssembly on a WebGPU canvas. Rust applications can use the runtime directly.
These guides document the current 0.4 source and its platform limits.
Application commands run from a generated project; contributor commands name
the repository root explicitly.

## Start here

1. [Getting started](getting-started.md): prerequisites, CLI projects, native
   and browser runs, and the checked-in Rust and TSX templates.
2. [Argui CLI](cli.md): 0.4 installation, every application command, build
   targets, tests, and component installation.
3. [Crates and features](crates.md): which Rust dependencies to add and how
   optional Cargo features affect an application.
4. [Architecture](architecture.md): ownership, reconciliation, invalidation,
   rendering, and platform boundaries.
5. [Solid and React host contract](solid-react-native.md): the transaction
   bridge, identity, state, and browser versus desktop capabilities.
6. [Gallery](../apps/gallery/README.md): runnable Solid and React controls.

## Build an interface

| Need | Guide |
| --- | --- |
| Custom components and reactive state | [Custom components and state](ui/custom-components.md) |
| Sizing, flex, grid, RTL | [Layout](ui/layout.md) |
| Native scrolling and virtualization | [Scrolling](ui/scroll.md) |
| Paint, borders, shadows, colors | [Styling](ui/styling.md) |
| Transitions, loops, and frame cost | [Animation](ui/animation.md) |
| Widget exports and state contracts | [UI and widgets](ui/README.md) |
| Focus, events, input, semantics | [Interaction](ui/interaction.md) · [Accessibility](ui/accessibility.md) |
| Theme runtime and tokens | [Theme](ui/theme.md) |
| Application assets | [Icons and media](ui/icons.md) |

## Windows and platform services

Start with [desktop windows](platform/desktop.md) for monitors, logical and
physical coordinates, zoom, resizing, movement, decorations, transparency,
input regions, native popups, and backend limits. Then use the focused guides:

- [Application setup](platform/application.md) and
  [safe areas](platform/window-insets.md)
- [Desktop backdrops](platform/desktop-backdrops.md) and
  [popups beyond the window](platform/desktop.md#popups-beyond-the-window)
- [File picker](platform/file-picker.md), [WebView](platform/webview.md), and
  [updater](platform/updater.md)
- [Android and iOS boundary](native-mobile.md)

## Engine and maintainers

- [Models and lifetime](runtime/models.md) and [owned tasks](runtime/tasks.md)
- [Rendering primitives](rendering/primitives.md), [text](rendering/text.md),
  [damage](rendering/damage.md), [compositor](rendering/compositor.md),
  [effects](rendering/effects.md), and [GPU canvases](rendering/gpu-canvas.md)
- [Internationalization](i18n.md) and [native automation](automation.md)
- [Repository structure](repo/structure.md), [development](contributing/development.md),
  [code quality](contributing/code-quality.md),
  [Linux graphical testing](contributing/linux-testing.md), and
  [releases](contributing/releases.md)

The current API is defined by the Rust schema and the generated Solid and
React JSX declarations.
