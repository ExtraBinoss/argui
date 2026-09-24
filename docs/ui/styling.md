# Conditional and responsive styling

Argui uses typed Rust values rather than a CSS parser or cascade. Flex, Grid,
intrinsic sizing, wrapping, min/max constraints, and alignment already adapt to
available space automatically. Container queries are reserved for a deliberate
change of presentation that normal layout cannot infer.

## Conditions

`Element::when` accepts any `StyleCondition`. State and container predicates can
be nested without changing the element tree:

```rust
let panel = ContainerScopeId::new("inspector");
let condition = StyleCondition::all([
    VisualState::Hovered.into(),
    !StyleCondition::state(VisualState::Disabled),
    ContainerQuery::min_width(panel, 420.0).into(),
]);

let element = Element::container([]).when(
    condition,
    StylePatch::new()
        .set(property::Opacity, 0.92)
        .set(property::Transform, Transform2D::IDENTITY.scale(1.02, 1.02)),
);
```

Rules are evaluated in declaration order. A later value replaces the same typed
property and leaves every unrelated value intact. Explicit motion bindings are
the final composition layer.

`StylePatch` covers quad paint, text and vector color, transforms, layers,
shadows, scroll, registered effect parameters, individual layout values, and a
complete `LayoutStyle`. A complete layout replacement is discrete; numeric
layout properties can use a transition and retain their existing Taffy node.

## Named containers

`Element::container_scope` defines a container by identity. A query on that node
or a descendant uses the nearest active scope with that identity, so nested
reusable controls do not leak dimensions into one another.

```rust
let scope = ContainerScopeId::new("toolbar");
let row = Element::row(actions);
let mut column = row.style.clone();
column.flex_direction = FlexDirection::Column;

let toolbar = Element::container([
    row.when(
        ContainerQuery::max_width(scope, 360.0),
        StylePatch::new().layout(column),
    ),
])
.container_scope(scope);
```

The layout engine measures, resolves matching rules, and repeats only when a
layout property changed. Stable node identities and the retained Taffy tree are
preserved. Four passes bound the work; a repeated size state or an unresolved
cycle returns `LayoutError::NonConvergentContainerQueries` instead of presenting
an unstable frame.

## Hit geometry

Paint does not define interaction geometry implicitly. `HitTestStyle` selects
`Bounds`, `RoundedRect`, or `Ellipse`, optional per-edge slop, and one of four
pointer policies: `Auto`, `None`, `BoxOnly`, or `ContentsOnly`. This supports
large touch targets, circular controls, interaction-transparent decoration, and
composite controls without invisible blocking boxes.

Paint-only conditions rebuild the display list and reuse layout and shaped
text. Scroll conditions reuse layout and update translated geometry. Layout
conditions recompute retained Taffy nodes. Idle conditions schedule no frame.

## Text overflow

`TextOverflow::Clip` preserves the complete shaped line behind its clip.
`TextOverflow::Ellipsis` replaces overflowing grapheme clusters with `…` and
reshapes only when the available width changes. Ellipsis applies to unwrapped
text; wrapped text continues to use its selected `TextWrap` strategy. The
`Element::text_overflow` builder configures text and text editors uniformly.
Single-line widget placeholders use ellipsis, while entered values retain the
normal horizontally scrolling editor behavior.

## Themes

`argui-theme` contains no global singleton. `Theme<T>` owns typed light and dark
values and resolves them with `ThemeMode::Light`, `Dark`, or `System`. Any
application data can be themed; TSX components can provide their own tokens.

Theme colors are authored in sRGB and stored internally as linear sRGB. See
[Color](../rendering/primitives.md#color) for the renderer-wide contract.

The runtime exposes `WindowEnvironment` through `Context::environment()`. It
contains the effective color scheme, reduced-motion preference, and
high-contrast preference, plus optional shared `ThemeOverrides`. It is `Clone`;
an environment without overrides allocates no token map. Components that read it are retained and rebuilt when
the environment changes; unrelated component caches remain valid.

`Context::entity_in(&entity, environment)` scopes an environment to a mounted
child and its descendants. `ThemeOverrides` holds typed color and number
overrides for applications that expose theme editing.

System mode uses Winit theme notifications on Windows, macOS, and Web. Linux
reads and continuously watches the XDG desktop portal. An unknown or explicitly
neutral system preference resolves to Light. `PreferenceOverrides` always wins
and is applied before the first visible frame.

SVG icons using `currentColor` can share cached alpha masks while
`Element::vector_color` supplies the color per instance. Theme changes repaint
existing vectors without duplicating or rerasterizing their assets.
