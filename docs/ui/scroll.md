# Scrolling and virtual lists

Use `scrollView` for a bounded viewport. Give it a resolvable height for
vertical scrolling or width for horizontal scrolling, either directly or
through a bounded flex parent with `grow={1}`. The content must exceed that
bound. A `scrollView` retains every child; use `VirtualList` when mounting all
rows would be expensive.

## A scroll viewport in TSX

`scrollY` is enabled and `scrollX` disabled by default. The native scrollbar
appears when content overflows. `scrollbarVisible={false}` hides it without
disabling wheel, touchpad, or other scroll input.

```tsx both
<scrollView width="100%" height={160} scrollY={true}
  scrollbarWidth={6} scrollbarHoverWidth={10}
  scrollbarThumbColor="#64748b" scrollbarHoverColor="#2563eb">
  <column width="100%" gap={4}>
    {Array.from({ length: 10 }, (_, index) =>
      <rectangle width="100%" height={32} shrink={0}
        padding={{ start: 8, end: 8 }} background="#f1f5f9">
        <text color="#0f172a">{`Row ${index + 1}`}</text>
      </rectangle>)}
  </column>
</scrollView>
```

The track and thumb belong to the same native viewport as scrolling. For the
vertical bar, `scrollbarSide` selects `"left"` or `"right"`;
`scrollbarEndInset` controls its top and bottom ends. The thumb's resting,
hovered, and dragged colors are `scrollbarThumbColor`,
`scrollbarHoverColor`, and `scrollbarPressedColor`. The track uses
`scrollbarTrackColor`. `scrollbarHoverWidth` changes the draggable geometry
with a native 140 ms width transition. Set it equal to `scrollbarWidth` for a
fixed width. `scrollMomentum` ranges from 0 (direct) to 1 (longer glide); an
omitted value uses the native default. The active [scrollbar example](../../apps/gallery/src/solid/scrollbar-page.tsx)
lets you compare these controls.

`onScroll` can report position changes to application code, but input,
clipping, scrollbar drag, sticky positioning, and movement are handled by the
native engine. A scroll callback is unnecessary just to move content.

## The Rust viewport

Rust elements express overflow on their layout style and scrolling policy in
`ScrollConfig`. This example has 256 logical pixels of rows inside a 160 pixel
viewport:

```rust
use argui_ui::{Axes, Element, Overflow, ScrollConfig, length};

let content = Element::column((1..=8).map(|index| {
    Element::text(format!("Row {index}"))
        .height(length(32.0))
        .shrink(0.0)
}));
let viewport = Element::column([content])
    .height(length(160.0))
    .overflow(Axes { x: Overflow::Hidden, y: Overflow::Auto })
    .scroll_config(ScrollConfig::default());
```

`ScrollConfig` sets axes, wheel polarity and line size, multiplier, physics,
overscroll, chaining, anchoring, and an optional `ScrollbarStyle`. Default
physics is light inertia and overscroll is clamped. For nested regions,
`ScrollPropagation::Chain` passes only unconsumed movement to an ancestor;
`Contain` stops that chain, and `None` clamps and consumes it. Native scroll
offsets are retained by stable node identity. `Position::Sticky` follows the
nearest scroll viewport while its siblings move.

## Large lists

`VirtualList` from `@argui/widgets/solid` or `@argui/widgets/react` renders
only a native-requested visible range with overscan. Give it a viewport bound,
an extent estimate, and a stable `itemKey` derived from data identity rather
than the row index. The key is separate from a native `id`.

```tsx solid
import { VirtualList } from '@argui/widgets/solid'

const records = Array.from({ length: 1000 }, (_, index) => ({
  id: `record-${index + 1}`,
  name: `Record ${index + 1}`,
}))

export function Records() {
  return <VirtualList count={records.length} width="100%" height={240}
    estimate={40} variable={false} overscan={4}
    itemKey={(index) => records[index]!.id}
    renderItem={(index) =>
      <row width="100%" height={40} alignItems="center" padding={8}>
        <text color="#0f172a">{records[index]!.name}</text>
      </row>}
  />
}
```

Use `variable={false}` only when the rendered row extent really matches
`estimate`. The default variable mode measures mounted rows and corrects the
offset as estimates become known. Lists with at most 12 items intentionally
mount every row. Increase `overscan` if rapid scrolling exposes the edge of
the mounted range, while keeping that range bounded. After inserts, removals,
or reordering in the middle, increment `dataVersion` so the native measurements
are reset. The active [scrolling example](../../apps/gallery/src/solid/virtual-list-page.tsx)
shows both kinds of viewport.

The equivalent Rust API is `VirtualList::fixed` or `VirtualList::variable`:

```rust
use argui_ui::{Element, VirtualList};

let list = VirtualList::fixed(1000, 40.0, 240.0);
let viewport = list.build("records", 0.0, |index| {
    Element::text(format!("Record {}", index + 1))
});
```

`VirtualList::scroll_to` computes an offset for an index. For ordinary
elements, `ScrollRequest::reveal` or `ScrollRequest::offset` can be sent with
`argui_runtime::Context::scroll`; reveal requests traverse nested scroll
ancestors. Focusing a control also reveals it. Direct input interrupts a
smooth request.

## Scroll edge treatments

`shadowWidth` on `scrollView` fades content only at edges with hidden content;
zero disables it. `shadowIntensity` controls its strength. The
`ScrollShadow` widget wraps that native behavior and chooses a size from the
widget theme. Keep the viewport at the full surface width and put content
padding inside it if the fade and scrollbar should reach the surface edge.
For Rust effects, enable `argui-effects`' `scroll` feature, register
`argui_effects::registry()` with the renderer, then attach
`argui_effects::EdgeFade::new(20.0).scroll()` through
`ScrollConfig::effect`. `EdgeShadow` paints a chosen color instead of fading
alpha. Effects use scroll metrics at paint time; they require an offscreen
filter while active.
