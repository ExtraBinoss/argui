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
| Model, task, lifecycle, frame scheduling | `argui-runtime` | facade re-export in `argui` |
| Reusable control | `argui-widgets` | gallery page and feature routing |
| End-to-end example | `app_examples/` | website catalogue entry |

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

## Adding a widget

1. Put the controlled state and view builder in `crates/argui-widgets/src/`.
2. Return actions or next state from input handling; the application owns the
   final mutation.
3. Add semantics, keyboard behavior, focus, disabled state, and reduced-motion
   handling as part of the widget contract.
4. Gate the module in `argui-widgets/Cargo.toml` and route the corresponding
   `widget-*` feature through `crates/argui/Cargo.toml`.
5. Add behavior tests under `crates/argui-widgets/tests/` with the matching
   source path.
6. Add a Widget Gallery page or focused application example. Document the API
   in the [widget catalogue](../widgets/shadcn.md).

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

Default builds stay small. Add optional functionality behind a feature in the
crate that owns it, then expose a facade feature only when application code
needs it. Use Cargo target dependencies for OS libraries. Android and iOS entry
crates stay explicit dependencies outside `argui --all-features`.

Compile the actual affected targets. A successful Linux build does not validate
macOS, Windows, Android, iOS, or WebAssembly code selected by `cfg`.

## Edit loop

Keep the feature set stable during the loop:

```sh
cargo fmt --all
cargo nextest run -p CRATE --all-features
cargo clippy -p CRATE --all-targets --all-features -- -D warnings
```

For a UI change, run the smallest relevant gallery scenario and inspect its PNG
on the private display. For performance work, establish a reproducible workload
before changing code and keep raw results under `docs/performance/data/`.

When implementation is complete, follow the one-time final gate in
[code quality](code-quality.md#final-gate).

## Keep documentation close to the contract

- Public functions and methods get Rustdoc at their definition.
- The documentation website owns tutorials and runnable examples.
- `docs/` owns architecture, lifecycle, platform limits, and contributor work.
- Raw benchmark data stays machine-readable; the performance guide states only
  conclusions supported by it.

Update the owning document in the same change as the behavior. Remove outdated
text instead of appending a correction below it.
