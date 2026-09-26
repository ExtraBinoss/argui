# Layout in Rust, Solid, and React

Argui uses the same retained Flexbox and Grid engine for Rust elements and
TSX primitives. A parent chooses how its children are arranged. Use `row` for
horizontal composition, `column` for vertical composition, and `grid` when
items need tracks or spans. Use `container` for a neutral box and `scrollView`
for a bounded scrolling viewport.

## Size and remaining space

`width` and `height` are preferred sizes in logical pixels. A flex item may
shrink below a preferred size when its parent has less room. A percentage such
as `width="50%"` is relative to the containing block; it does not mean the
remaining half after siblings have taken space. `grow={1}` consumes remaining
flex space. For a rigid item, set `shrink={0}` and a preferred size, or give it
equal minimum and maximum bounds when the size must never change.

```tsx
<row gap={12} width="100%">
  <column width={240} shrink={0}>Navigation</column>
  <column grow={1} minWidth={0}>Contenu</column>
</row>
```

`minWidth={0}` lets the second child shrink below its text's intrinsic width.
Without it, long unbroken content may overflow. `auto` uses content and parent
rules. `fill` is deliberately absent: a percentage and flex growth have
different meanings. `minWidth`, `maxWidth`, `minHeight`, and `maxHeight` give
explicit bounds in logical pixels, percentages, or `auto`; `aspectRatio` is
the preferred width divided by height.

## Spacing and direction

The parent normally owns `gap` between children and `padding` inside its box.
Use `margin` for an individual child's outer spacing. An insets value can be a
single number or an object with `top`, `right`, `bottom`, `left`, `start`, and
`end`. `start` and `end` follow `directionScope` (`ltr` or `rtl`). Use physical
or logical horizontal sides in a given value; the native boundary rejects a
mixture of the two conventions.

```tsx
<column directionScope="rtl" padding={{ start: 20, end: 12, top: 8 }} gap={8}>
  <text>مرحبا</text>
</column>
```

`position="absolute"` removes an element from normal flow. Give it an
`inset` value to place it against its containing box. `position="sticky"`
keeps the element against its nearest scroll viewport. A `transform` changes
paint and hit geometry, but reserves no extra layout space. Popovers should
use an `id` anchor instead of JS calculations of screen coordinates.

## Grid and responsive rules

`gridColumns` and `gridRows` take typed arrays. A number is a fixed logical
pixel track, `auto` and percentages use their normal layout meanings, and
`{ fr: n }` takes a fraction of free space. `minmax` bounds one track; a
repeat group can use a fixed count or `autoFit`/`autoFill`.

```tsx
<grid
  gap={12}
  gridColumns={[
    { repeat: { count: 'autoFit', tracks: [{ minmax: { min: 180, max: { fr: 1 } } }] } },
  ]}
>
  {cards}
</grid>
```

An item can set `gridColumnStart` and `gridColumnSpan` (and the row
equivalents). For deliberate presentation changes, `containerScope` names a
measurement scope and `containerRules` applies typed layout overrides when an
ancestor scope meets a size or orientation condition. The Rust engine bounds
responsive convergence and reports cycles rather than showing an unstable
frame. Prefer ordinary flex/grid adaptation when it already gives the wanted
result.

## Scrolling and inspection

`scrollView` is the public viewport primitive. Set a resolvable `height` or
`maxHeight` for vertical scrolling, or place it in a bounded flex parent with
`grow={1}`. `scrollX` and `scrollY` select axes. Scrolling, sticky movement,
scrollbar interaction, and virtual list windowing stay in the native engine;
there is no JS callback for each movement frame.

In development, the inspector reports preferred and bounded sizes, flex
growth and shrink, computed geometry, and notes for likely mistakes such as a
percentage against an unresolved parent or a scroll viewport without a
vertical bound. These are diagnostics, not a second layout algorithm.

Rust builders use idiomatic `snake_case` method names for these same concepts;
public schema and TSX property names use `camelCase`.
