# Styling and responsive rules

Argui uses typed values, not a CSS cascade. TSX properties are validated by
the generated native schema; Rust `Element` builders and `StylePatch` use the
same engine. Paint, layout, and interaction geometry are separate choices.

## Surfaces and text in TSX

`rectangle` accepts a solid color or a typed gradient for `background`, plus
`border`, `radii`, `shadow`, and state paint. `container`, `row`, and `column`
are layout primitives with a solid `background`; use `rectangle` when the
surface needs a gradient or hover/press colors. `clip={true}` clips descendants
to the surface's bounds and `radii`; `radii` alone rounds the painted surface.

```tsx both
<rectangle width={240} height={96} padding={12} radii={12} clip={true}
  background={{ kind: 'linear', angle: 0, space: 'oklab', stops: [
    { offset: 0, color: '#2563eb' },
    { offset: 1, color: '#7c3aed' },
  ] }}>
  <text color="#ffffff">Gradient card</text>
</rectangle>
```

Gradient stops are ordered offsets from 0 to 1. A radial brush instead takes
`center` and `radius` points; the accepted shapes are defined in the
[host protocol](../../packages/host/src/protocol.ts). Color strings accept
hex, `oklch(...)`, `rgb(...)`, and `rgba(...)`. A palette name such as
`blue-500` is not a color literal.

`hoverBackground`, `pressedBackground`, `focusBorderColor`, and native
transitions are available on `rectangle`. Put hover/press paint on the same
surface that owns the interaction. For text, use either children or the `text`
property, not both. Native `<text>` has its own text properties, such as
`color`, `fontSize`, `noWrap`, `lineClamp`, and `textOverflow`; it does not
accept a container's `margin` or `grow`. Wrap it in a layout primitive when
those are needed. An unwrapped line can use
`<text width={160} noWrap={true} textOverflow="ellipsis">A long label that will not fit</text>`;
`ellipsisStart` and `ellipsisMiddle` are also available.

For application colors, supply one theme runtime at the root. Widgets consume
semantic tokens; raw `<text>` still needs an explicit `color`. The
[theme guide](theme.md) covers Neutral tokens, light/dark/system selection,
local overrides, and contrast checks.

## Rust conditional styles

`Element::when` attaches a `StylePatch` to a `StyleCondition`. A condition can
test the element's visual state, a named state scope, a container query, or an
`all`/`any`/`not` combination. Later matching rules replace an earlier value
for the same typed property and leave unrelated properties alone.

```rust
use argui_animation::{Duration, Transition, Tween};
use argui_ui::{
    Color, Element, Interaction, StylePatch, StyleTransition,
    VisualState, property,
};

let card = Element::container([Element::text("Hover me")])
    .background(Color::WHITE)
    .interaction(Interaction::default())
    .when(
        VisualState::Hovered,
        StylePatch::new().set(
            property::BackgroundColor,
            Color::srgba(0.82, 0.90, 1.0, 1.0),
        ),
    )
    .transition(StyleTransition::new(Transition::tween(
        Tween::new(Duration::from_millis(160)),
    )));
```

`StylePatch` can set individual typed paint, transform, layer, scroll, effect,
and layout properties, or replace a complete `LayoutStyle`. Numeric compatible
values can interpolate; a complete layout replacement is discrete. A direct
`Element::bind` motion takes final precedence over a transition on the same
property. See [animation](animation.md) for update costs and reduced motion.

## Container queries

Flex and grid already respond to available space. Use a named container query
when the presentation itself must change at a threshold. `containerScope`
names the measured ancestor; each `containerRules` entry names that scope,
conditions on its width, height, or orientation, and a supported layout
override. The rule's `style` supports grid tracks, width/height, gap,
grow/shrink, `alignItems`, and `justifyContent`—it is not an arbitrary
style object.

```tsx both
<container width="100%" containerScope="cards">
  <grid width="100%" gap={8} gridColumns={[{ fr: 1 }]}
    containerRules={[{
      scope: 'cards',
      when: { minWidth: 480 },
      style: { gridColumns: [{ fr: 1 }, { fr: 1 }] },
    }]}>
    <rectangle height={48} background="#dbeafe" />
    <rectangle height={48} background="#dbeafe" />
  </grid>
</container>
```

Rust uses `ContainerScopeId`, `ContainerQuery`, and `Element::when` with a
layout patch. The engine remeasures when a layout rule changes, retaining node
identities. If a query causes a size cycle or does not settle within four
passes, layout returns `LayoutError::NonConvergentContainerQueries`; avoid a
rule that repeatedly flips the queried container's own width. The
[layout guide](layout.md) covers natural flex and grid sizing first.

## Hit geometry and text overflow in Rust

Painted shape does not implicitly choose pointer geometry. Rust
`HitTestStyle` offers bounds, rounded rectangle, or ellipse hit shapes,
per-edge slop, and `Auto`, `None`, `BoxOnly`, or `ContentsOnly` pointer policy.
Set it explicitly for a circular target or an interaction-transparent
decoration. `Element::text_overflow` accepts `TextOverflow::Clip` or
`TextOverflow::Ellipsis(position)`; ellipsis applies to an unwrapped line.
Wrapped text follows its wrapping strategy. A transform updates pointer
geometry without changing the element's reserved layout size.
