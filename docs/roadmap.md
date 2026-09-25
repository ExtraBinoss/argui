# Roadmap

Current features are documented in the [guides](README.md). This page tracks
work that remains open; it is not a release schedule.

## Platform coverage

- [ ] Complete Android support: validate the packaged Solid/React TSX gallery on
  emulator and device, finish platform services, then ship owner-signed APK/AAB
  artifacts.
- [ ] Complete iOS support: provide a native sample shell for `argui-runtime`'s iOS entry, then
  validate startup, input, lifecycle, and accessibility in Simulator and on
  device before shipping signed archives to TestFlight.
- Extend native Windows and macOS interaction, installation and rendering tests
  beyond their complete-workspace CI compilation.
- Validate real screen readers and IMEs across desktop and web, including
  bidirectional text, modal focus and virtualized controls.
- Add native Wayland popovers behind the existing surface preference API.
- Expand Android/iOS packaging CI with emulator and physical-device smoke tests.

## TSX developer experience

- [ ] Build first-class DevTools for Solid and React applications using
  `@argui/widgets` for inspector panels and controls. Integrate with the native
  QuickJS hot-reload loop and expose the retained tree, semantics, runtime
  diagnostics, and useful layout/rendering state.

## Application APIs

- A public application test harness for input, focus, lifecycle and asynchronous work.
- Typed drag-and-drop and native application menus connected to scoped actions.
- Geometry transitions for insertion, removal and reordering.
- A documented compatibility policy as the public API stabilizes.

Retained custom WGPU viewports are complete for Argui-owned devices and
surfaces; see [GPU canvases](rendering/gpu-canvas.md). A lower-level API for
embedding externally owned WGPU instances, devices or surfaces remains outside
the high-level runtime and would require a separate ownership design.

Performance work follows [measured workloads](performance/optimizations.md).
