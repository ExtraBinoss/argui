# Solid and React host contract

Solid Universal and React describe Argui elements through the same transaction
protocol. `@argui/host` sends typed operations to `argui-host`; `argui-schema`
defines their names and values. The Rust host reconciles those operations into
the retained `UiTree`. JavaScript does not calculate layout, shape text, paint,
or own the accessibility tree. The [architecture guide](architecture.md)
describes the engine boundary.

The desktop gallery embeds QuickJS. Bun builds and tests its TSX bundles; it
is not the application runtime. The browser gallery initializes the WASM host
and renders to a WebGPU canvas. `apps/gallery/src/web-mount.ts` bridges the
same generated ABI to the browser host. A Rust application can use
`argui-runtime` without either TSX adapter or QuickJS.

## Identity and state

Framework `key` preserves reconciliation identity. Native `id` is optional and
addressable for popup anchors, accessibility relations, and tests. Keep both
stable across updates. Controlled widgets receive `value` and
`onValueChange`; overlays use `open` and `onOpenChange`. See the
[widget guide](ui/README.md) for the current component exports.

Each framework adapter mounts one shared root and disposes it when the host
ends. The CLI templates show the exact [Solid](../crates/argui-cli/assets/templates/solid-main.tsx)
and [React](../crates/argui-cli/assets/templates/react-main.tsx) bootstrap
functions. The native gallery uses the corresponding
[Solid](../apps/gallery/src/solid/main.tsx) and
[React](../apps/gallery/src/react/main.tsx) entry points.

## Platform boundary

The browser host needs WebGPU. Native platform services such as native
outside-window popups, native window movement, global shortcuts, and screen
overlays require the appropriate desktop backend and are not Web capabilities.
Feature availability must be checked before showing a demonstration that
requires them. The [desktop guide](platform/desktop.md) describes the backend
limits; [native mobile](native-mobile.md) records the Android shell and iOS
integration boundary.

The packaged Android gallery is a QuickJS Solid app. The repository does not
include an iOS application shell or Xcode project. Native accessibility,
rendering, and input support on one platform do not establish screen reader
or interaction behavior on another; validate the target device and backend.

## Checks

`bun run check:ts` checks generated TSX types. `bun run test:ts` builds both
gallery adapters and runs their host interaction tests. A typecheck alone does
not prove that a native transaction mounts; run the owning host tests as
well. For actual graphical checks on Linux, use the
[private-display procedure](contributing/linux-testing.md) and inspect its
saved captures.
