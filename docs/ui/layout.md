# Layout

Argui lays out Rust `Element` trees and Solid/React TSX primitives with the
same retained Taffy engine. The parent controls its children's arrangement:
`row` and `column` use flex layout, `grid` uses tracks, and `container` is a
neutral block. Dimensions and spacing are in logical pixels unless a value is
a percentage.

## Preferred size, growth, and constraints

`width` and `height` are preferred sizes. Flex children can shrink below them.
`grow` takes remaining space after preferred sizes and gaps have been measured;
`width="50%"` measures half the containing block instead. Use `shrink={0}` for
a rigid item, and `minWidth={0}` when a flexible item with long content must
shrink. `auto` leaves sizing to content and layout. There is no `fill` value.

```tsx both
<row width="100%" gap={12}>
  <rectangle width={180} shrink={0} background="#e2e8f0" padding={12}>
    <text color="#0f172a">Navigation</text>
  </rectangle>
  <rectangle grow={1} minWidth={0} background="#dbeafe" padding={12}>
    <text color="#0f172a">This content wraps as the available width changes.</text>
  </rectangle>
</row>
```

The equivalent Rust builders use `snake_case`. `length` expresses logical
pixels; Rust `percent(0.5)` expresses 50%.

```rust
use argui_ui::{Element, length};

let navigation = Element::container([Element::text("Navigation")])
    .width(length(180.0))
    .shrink(0.0);
let content = Element::container([Element::text("Content")])
    .grow(1.0)
    .min_width(length(0.0));
let row = Element::row([navigation, content])
    .width(length(640.0))
    .gap(12.0);
```

`minWidth`, `maxWidth`, `minHeight`, and `maxHeight` accept pixels,
percentages, or `auto` in TSX. `aspectRatio` is preferred width divided by
height. A percentage needs a resolvable containing size; if the parent's size
depends on that child, give the parent a bound or use flex growth instead.

## Spacing, direction, and position

Put repeated spacing on the parent with `gap`, and inner spacing on a surface
with `padding`. `margin` is outer spacing for one child. TSX insets accept a
number or an object with `top`, `bottom`, and either physical `left`/`right` or
logical `start`/`end`. Do not mix physical and logical horizontal sides in one
object. `directionScope="rtl"` reverses `start` and `end` for descendants.

```tsx both
<container width={240} height={80} directionScope="rtl" background="#e2e8f0">
  <rectangle width={120} height={28} position="absolute"
    inset={{ start: 12, top: 8 }} background="#2563eb">
    <text color="#ffffff">From the start edge</text>
  </rectangle>
</container>
```

An absent positioned inset stays `auto`; it is not zero. `position="absolute"`
removes the child from normal flow. `position="sticky"` constrains it against
the nearest scroll viewport. `zIndex` orders siblings within a window layer.
`transform` changes painting and pointer geometry but reserves no additional
layout space. See [scrolling](scroll.md) for a bounded viewport and
[styling](styling.md) for clipping and pointer hit shapes.

## Grid tracks

`gridColumns` and `gridRows` take typed arrays. A number is a fixed logical
pixel track; `"auto"` and percentages are also accepted. `{ fr: 1 }` takes a
share of free space. `minmax` bounds a track, and `repeat` accepts a fixed
count, `autoFit`, or `autoFill`.

```tsx both
<grid width="100%" gap={8} gridColumns={[
  { repeat: { count: 'autoFit', tracks: [
    { minmax: { min: 140, max: { fr: 1 } } },
  ] } },
]}>
  <rectangle height={40} background="#dbeafe" />
  <rectangle height={40} background="#dbeafe" />
  <rectangle height={40} background="#dbeafe" />
</grid>
```

For explicit placement, `gridColumnStart` and `gridRowStart` are one-based
grid lines; `gridColumnSpan` and `gridRowSpan` count occupied tracks. Prefer
natural flex or grid adaptation before adding a [container query](styling.md#container-queries).

The active gallery's [layout scenarios](../../apps/gallery/src/solid/layout-scenarios.tsx)
show resizing, overflow, scrolling, adaptive grid tracks, and logical insets.
