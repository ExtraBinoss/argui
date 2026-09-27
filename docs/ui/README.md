# UI and widgets

Argui has native Rust UI primitives and TSX adapters for Solid and React. The
`@argui/widgets` package composes those primitives into themed controls. Choose
the guide for the layer you are changing:

- [Widgets and controls](controls.md): component inventory, state, and TSX use.
- [Custom components and state](custom-components.md): compose `rectangle`,
  `row`, `column`, and `text`, with two live WebAssembly examples.
- [Interaction](interaction.md): native events, focus, text input, and popups.
- [Theme](theme.md): root provider, semantic colors, variants, and overrides.
- [Accessibility](accessibility.md): names, roles, relations, and actions.
- [Layout](layout.md), [scrolling](scroll.md), and [animation](animation.md):
  geometry and motion used by both the Rust and TSX layers.

The generated public TSX contracts live in
[`packages/solid/src/jsx.generated.ts`](../../packages/solid/src/jsx.generated.ts)
and [`packages/react/src/jsx.generated.ts`](../../packages/react/src/jsx.generated.ts).
Widget props live in [`packages/widgets/src/shared/types.ts`](../../packages/widgets/src/shared/types.ts)
and [`new-controls.ts`](../../packages/widgets/src/shared/new-controls.ts).
