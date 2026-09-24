# Development guide

This guide covers the shortest path from an issue to a reviewable Argui change.
Read [architecture](../architecture.md) first when the change crosses crates.

## Find the owner

Start with the type or behavior, then edit the crate that owns it:

| Change | Primary crate | Common follow-up |
| --- | --- | --- |
| Geometry, color, input value type | `argui-core` | consumers that translate the type |
| Element, style, focus, event, semantics | `argui-ui` | layout or runtime integration |
| Flexbox/Grid calculation | `argui-layout` | UI invalidation test |
| Text shaping, bidi, cursor geometry | `argui-text` | renderer glyph preparation |
| Display-list primitive | `argui-paint` | WGPU implementation in `argui-render` |
| Window or OS capability | `argui-platform` | command/result routing in `argui-runtime` |
| Model, task, lifecycle, frame scheduling | `argui-runtime` | applications using the runtime APIs |
| TSX host contract or transaction | `argui-schema` / `argui-host` | matching package under `packages/host` |
| Reusable TSX control | `packages/widgets` | Solid/React adapter and gallery page |
| TSX application example | `apps/gallery/` | gallery page or package test |

Use `rg` before adding a type. Geometry, colors, keys, actions, and window
commands already have shared representations; parallel versions create
conversion code and inconsistent behavior.

## Follow the data flow

For an interaction change, trace:

```text
platform event -> runtime normalization -> UI dispatch -> model update
               -> reconciliation -> layout/text/paint invalidation -> frame
```

For a rendering change, keep the description and execution separate:

```text
Element builder -> argui-paint command -> argui-render resource/pipeline
```

The Solid and React adapters translate TSX into host transactions. Keep
framework-specific code in `packages/`; add a Rust engine primitive only when
the existing host schema and retained elements cannot express the behavior.

For an OS integration, keep native handles inside the platform adapter and
return an ordinary command result to the application.

## Model code

`Render::render` describes the current view. Event listeners mutate the owning
model and call `cx.notify()` when the description may change. Use narrower
requests when the tree stays identical:

- `request_paint()` for paint data produced outside a model rebuild;
- `request_animation_frame()` while an animation remains active;
- `scroll()` for a retained scroll request;
- `request_focus()` for a stable focus target.

Keep durable state in an `Entity<T>`. Keep presentation-local handlers, tasks,
and caches in its `Mount<T>` or `Context<T>`. Do not store Winit or WGPU handles
in application models.

Use stable keys for reordered siblings, virtualized rows, focus targets, and
elements observed after layout. A key is identity, not visible copy.

## Adding a TSX widget

1. Put framework-neutral types and behavior in `packages/widgets/src/shared/`.
2. Add the Solid or React view under the corresponding `src/solid/` or
   `src/react/` directory and export it from that package entry point.
3. Use the shared host primitives and contracts. If an engine primitive or
   transaction is genuinely missing, extend `crates/argui-schema` and
   `crates/argui-host` before adding framework-specific workarounds.
4. Add or update tests in the owning TypeScript package, and cover gallery
   behavior in `apps/gallery/tests/` when it depends on the native host.
5. Add a page to `apps/gallery/` when the control needs an interactive example.
   The [gallery guide](../../apps/gallery/README.md) documents the package
   imports and native development loop.

Do not put network requests, persistence, or application validation inside a
generic widget.

## Adding an engine capability

Change the lowest owning crate first. Add the next layer only when translation
is required. A new paint primitive, for example, normally needs:

1. an immutable command and validation in `argui-paint`;
2. collection from an element or custom element in `argui-ui`;
3. batching or pipeline execution in `argui-render`;
4. renderer-neutral tests before a GPU capture test.

State the invalidation cost in the API: whether a value changes layout, shaped
text, paint, hit geometry, or only semantics. Reuse retained resources and bound
all caches.

## Platform and feature gates

Keep default builds small. Add optional functionality behind a feature in the
crate that owns it, and let applications depend on that crate directly. Use
Cargo target dependencies for OS libraries. Android and iOS entry crates stay
explicit dependencies.

Compile the actual affected targets. A successful Linux build does not validate
macOS, Windows, Android, iOS, or WebAssembly code selected by `cfg`. The
repository's iOS engine cross-check does not build a sample iOS application;
the checked-in packaged gallery currently targets Android.

## Edit loop

Keep the feature set stable during the loop:

```sh
cargo fmt --all
cargo nextest run -p CRATE --all-features
cargo clippy -p CRATE --all-targets --all-features -- -D warnings
```

For a UI change, run the smallest relevant gallery scenario and inspect its PNG
on the private display. For performance work, establish a reproducible workload
before changing code and keep raw results under `target/performance/` while
investigating. Check in only the results needed to support a maintained guide.

For TSX changes, run `bun run check:ts` and `bun run test:ts`. The native
gallery's Solid/React hot reload is documented in the
[gallery guide](../../apps/gallery/README.md); run its desktop window through
the private-display helper when checking visuals.

When implementation is complete, follow the one-time final gate in
[code quality](code-quality.md#final-gate).

## Keep documentation close to the contract

- Public functions and methods get Rustdoc at their definition.
- The maintained native TSX gallery and its runnable examples live in
  `apps/gallery/`. The documentation site will be rebuilt separately.
- `docs/` owns architecture, lifecycle, platform limits, and contributor work.
- Raw benchmark data stays machine-readable; the performance guide states only
  conclusions supported by it.

Update the owning document in the same change as the behavior. Remove outdated
text instead of appending a correction below it.
