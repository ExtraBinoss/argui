# Custom elements

## Custom TSX primitives

A TSX application can register a native schema alongside Argui's built-ins.
Export the complete registry with `SchemaRegistry::contract()` using the Rust
`serde` feature, then generate both framework declarations from that JSON:

```sh
bunx --bun argui-generate-jsx contract.json src/generated
```

The command writes `react-custom.d.ts`, `solid-custom.d.ts`, and the exact
`contract.generated.json` fingerprint. Include the declaration matching the
application's framework in its TypeScript project and pass the same complete
registry to the native host. The generator refuses a custom contract that
changes or omits a built-in primitive. The host also validates custom property
names and values when loading the schema, so a generated type never replaces
native validation. Keep the JSON and host registration together: an ABI hash
mismatch stops mounting before a partially interpreted UI is displayed.

The Rust `CustomElement` layout/paint API below is separate from registering a
TSX primitive. A custom TSX primitive may adapt any retained `Element` built
with ordinary Argui facilities, including a `CustomElement`.

`Element::custom(properties)` installs a consumer-defined leaf;
`Element::custom_container(properties, children)` installs a custom layout with
ordinary Argui children. Both use `CustomElement` and the same retained layout,
paint, input and accessibility pipeline. No application-specific core variant,
second renderer or per-extension event loop is required.

The implementation supplies:

- `State` and `create_state`: scratch storage owned by the layout node, not shared
  between windows. Removal or a change of element type drops it.
- `layout_revision`: change this whenever intrinsic measurement changes.
- `paint_revision`: change this whenever the painted output changes. A paint-only
  update does not invalidate intrinsic measurement.
- `layout`: return logical content dimensions and an optional baseline. Negative or
  non-finite dimensions and invalid baselines produce a `LayoutError::Custom`.
- `prepare`: populate cached geometry for the current dimensions and revisions.
- `paint`: append renderer-neutral drawing commands. `context.quad` applies local
  positioning, ancestor transforms and clips. The element's usual effects and
  layers surround its drawing in the existing display list.

Use a stable element key. Duplicate sibling keys involving a custom element or
inside a custom container are rejected by layout. Revisions describe immutable properties: changing properties
without changing the corresponding revision violates the invalidation contract.
State is not application data; a full layout-engine reset (including asset-metric
replacement) can reconstruct it. Keep durable application data in an `Entity`.

## Example: timeline ruler

A timeline ruler is one use for a custom container: paint its grid, then place
ordinary Argui controls and transparent custom regions for clip resizing. A
clip region can handle drag, cancel, keyboard movement and accessible
increment/decrement actions through the same model update. Keep selection and
zoom in application state; expose the ruler's intrinsic width to layout so its
viewport can scroll horizontally.

## Layout contract

`layout(state, &mut dyn CustomLayoutContext) -> Result<CustomMeasurement, String>`
returns intrinsic **content-box** dimensions and an optional content baseline.
Dimensions and baseline must be finite and non-negative; baseline cannot exceed
height. The engine applies authored min/max sizes, border, padding and reserved
scrollbar gutters. Coordinates are logical pixels; the host owns DPI conversion
and final rounding.

- `available()`: offered content space; `None` means unbounded.
- `known_size()`: dimensions resolved by the parent, excluding insets.
- `child_count()`: declared children, in declaration order.
- `measure_child(index, available)`: measure a direct child through the engine.
- `place_child(index, bounds)`: place its border box relative to the content origin.

Every child must be placed exactly once per callback, including hidden children.
Repeated measurement is allowed. Invalid indices, duplicate or missing placements,
non-finite coordinates and invalid lengths produce a contextual error. Failed
layout is not cached as a successful fallback; a valid replacement can recover
on the same engine. Children retain their normal min/max sizing constraints.

The engine may invoke layout for intrinsic measurement and again with final
content dimensions. Use `known_size` when placement depends on the resolved size;
do not assume one invocation per frame. Only direct children can be measured,
so extensions cannot request parent/ancestor layout recursively. Layout and
painting must not mutate models or dispatch application work; use listeners and
model transactions for application updates.

## Interactive and accessible regions

`Element::custom_region(key, interaction, semantics)` declares a transparent,
non-text-selectable region. Pass it among a custom container's children and place
it through the layout context. Add ordinary `.on(...)` listeners, action bindings,
focus styles or paint. Regions are ordinary retained UI nodes, not a parallel
identity registry, hit-test system or semantic tree.

Solid and React components can add the shared semantic properties directly to
any native primitive, including `rectangle` and `text`. See the
[accessibility guide](accessibility.md) for roles, states, key relations, focus
and accessible action callbacks.

Regions therefore use normal hit shapes, cursors, focus, gestures, capture,
keyboard/action routing and native/Web semantic updates. Use a group role on a
semantic container; a button/slider role intentionally represents a semantic leaf.
Regions and standard children share declaration/z-index ordering; the frontmost
hit wins. Portals retain their overlay policy. Transforms and ancestor clips apply
before hit testing. Capture continues outside bounds. Removal clears capture,
hover, focus and handlers through normal reconciliation.

Semantic-only edits invalidate semantics; interaction/visual-only edits refresh
paint and hit regions without relayout. Changing placement requires a new layout
revision. Changing prepared geometry or color requires a new paint revision.

## Inspection and verification

`LayoutEngine::custom_stats()` samples cumulative layout, preparation and paint
counts and the last offered/resolved constraints without invoking any phase.
DevTools includes these in custom-node summaries when requesting a snapshot;
there is no continuous snapshot allocation with inspection inactive. Regions
remain inspectable descendants with their own identities and bounds.

The [custom layout tests](../../crates/argui-layout/tests/custom.rs) cover state
lifetime, measurement and paint invalidation, invalid sizes, duplicate keys and
retained GPU canvas slots. Check host-specific pointer, focus and accessibility
behavior in the consuming application. Follow the [Linux testing procedure](../contributing/linux-testing.md)
and inspect saved captures; a successful interaction assertion alone does not
establish visual correctness.
