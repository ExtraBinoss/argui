# Roadmap

Current features are documented in the [guides](README.md). This page tracks
work that remains open; it is not a release schedule.

## Coming next

- **Hot reload:** shorten the edit-and-preview loop for Rust interfaces.
- **Internationalization:** application-level locale management and translated copy.
  Direction-aware layout and configurable widget labels already exist.

## Platform coverage

- Extend native Windows and macOS interaction, installation and rendering checks.
- Validate real screen readers and IMEs across desktop and web, including
  bidirectional text, modal focus and virtualized controls.
- Add native Wayland popovers behind the existing surface preference API.
- Expand CI across supported operating systems and feature combinations.

## Application APIs

- A public application test harness for input, focus, lifecycle and asynchronous work.
- Typed drag-and-drop and native application menus connected to scoped actions.
- Geometry transitions for insertion, removal and reordering.
- A documented compatibility policy as the public API stabilizes.

Performance work follows [measured workloads](performance/optimizations.md).
A future optional DSL must produce the same public elements without becoming
a dependency of the renderer or runtime.
