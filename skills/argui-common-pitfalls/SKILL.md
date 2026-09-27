---
name: argui-common-pitfalls
description: Read before creating an Argui component or using Argui in Rust, Solid, or React. Covers layout, scrolling, animation architecture, expressive motion, gradients, throbbers, theme contrast, identities, and the native contract.
---

# Argui v2 common pitfalls

Read this skill before each new Argui component or Argui usage. The contracts
needed for the common decisions are included here; the skill does not require
the repository documentation to be present. Check the generated Solid or React
JSX declaration for each primitive used. A TypeScript pass alone does not prove
the Rust host accepts the final tree: build and mount the actual adapter too.

## Framework contract

Argui's Rust engine retains the UI tree. Solid and React produce transactions
through `@argui/host`; `argui-schema` defines their names and wire values. The
host owns native element identity, layout, text, input, accessibility, paint,
and animation. TSX state changes describe target properties, not per-frame
pixels. A Rust application can use `argui-runtime` directly without a JS host.

`row`, `column`, `container`, `rectangle`, `grid`, `text`, `scrollView`,
`focusScope`, and `popup` are native primitives. Flex sizes have distinct
meanings: `width`/`height` are preferred sizes, percentage sizes resolve
against a parent bound, and `grow` receives free space. Give a scroll view a
resolvable viewport size. `key` is framework identity; optional native `id`
is the addressable identity used by anchors, relations, and tests. The same
retained tree and schema run through the desktop QuickJS host and browser WASM
host. A browser host needs WebGPU and has no native desktop window services.

Reusable controls are exported separately from `@argui/widgets/solid` and
`@argui/widgets/react`. Install one widget theme provider at the app root;
local scopes override only selected tokens. Built-in controls own their input,
focus, and semantic behavior. Application icons are supplied through slots or
assets; widgets do not impose an icon library.

## Common pitfalls

- Public schema and TSX names are `camelCase`; Rust identifiers stay idiomatic
  `snake_case`. Change the Rust schema first, regenerate both declarations,
  and sync the CLI SDK snapshot. Do not add aliases for removed names.
- Framework `key` controls reconciliation. Native `id` is optional and
  addressable for anchors, accessibility relations, and tests. Do not derive an
  ID from visible text or a list index. A virtual list still needs a stable
  item key independent of native `id`. In Solid, use `<For>` for a reactive
  navigation list with fixed native IDs; recomputing `.map()` from a signal
  can create a replacement node before the old ID is released.
- `width` and `height` are preferred sizes. A percentage measures the parent;
  `grow` takes remaining flex space. Use `shrink={0}` for a rigid item and
  `minWidth={0}` when long flex content must shrink. There is no `fill` size.
- Use `scrollView` with a resolvable viewport bound. A native scrollbar appears
  when content overflows and grows on hover by default; use
  `scrollbarVisible={false}` when a viewport needs no visible scrollbar.
  When customizing it, use the native scrollbar contract, including
  side, width and hover style; keep scroll, dragging and transitions native.
  Check that content actually overflows and the thumb is reachable. Do not
  place a decorative bar over a separate invisible scrollbar.
- A bounded `scrollView` still mounts every child. For a popup with many
  expensive rows, use `VirtualList` with stable item keys and a measured row
  estimate; lists of 12 items or fewer intentionally mount all rows. Check
  opening time and that the last item remains reachable after scrolling. Base
  the choice on mounted native nodes and measured latency, not item count alone.
- For an edge shadow, use `ScrollShadow` and let native scroll metrics reveal
  each edge only while content remains hidden there. Give the viewport the
  full width of the surface and put content padding inside it; otherwise the
  fade and scrollbar stop short of the edge. Use `scrollbarEndInset={0}` when
  the scrollbar must reach the viewport ends. Inspect both scroll limits in a
  native capture; a static gradient can leave a visible band with no overflow.
- A radius on the TSX root clips GPU content, not the native window decoration.
  On Wayland, inspect the actual title bar and bottom border before rounding
  the client surface: an independently drawn square decoration can leave dark
  corner wedges. Check the result at multiple UI zoom factors because TSX
  pixel radii scale with the content.
- Use `start`/`end` insets for RTL; do not mix logical and physical horizontal
  sides in one TSX value. An absent positioned inset means `auto`, unlike zero
  padding.
- `<text>value</text>` and `<text text={value} />` are alternative forms; do
  not set both. Use `children` for ordinary Button text and give an icon-only
  Button an accessible name. Native `<text>` does not accept container-only
  props such as `margin` or `grow`; put it inside a `container`, `row`, or
  `rectangle` when it needs those layout rules.
- Provide the theme runtime once at the root. A local `ThemeScope` supplies
  sparse overrides for a subtree. Raw `<text>` does not automatically read
  the widget theme: bind its `color` to the current text token, and verify
  contrast after switching both light and dark variants. Check hover, focus,
  placeholder and disabled colors as well. `WidgetTheme` exposes the official
  Neutral semantic roles (`background`, `foreground`, `primary`, `muted`,
  `sidebar*`, chart colors, etc.). Use those roles instead of hardcoded colors;
  `accent` is a subtle surface, not the primary action color. The default
  selection follows the system scheme, and Settings must offer System as a
  real third state. Color tokens and TSX color properties accept `#RGB(A)`,
  `#RRGGBB(AA)`, `oklch(...)`, `rgb(...)`, and `rgba(...)`; the native parser
  validates channels and alpha. A Tailwind name such as `blue-500` is not a
  color literal: provide its actual CSS color value. Controlled values use
  `value`/`onValueChange`; panels use `open`/`onOpenChange`. A controlled value
  without its callback is read-only. `Popover open` is stricter: TypeScript
  requires `onOpenChange`, and `defaultOpen` cannot accompany `open`.
- Keep interaction and paint on the same native control: `focusScope` owns
  keyboard activation and accessibility while its visible `rectangle` owns
  `hoverBackground`, `pressedBackground`, and focus paint. Verify that a real
  pointer hover and press change the rendered surface, not only callbacks.
  Anchor popup content to a stable native `id` and let the popup placement
  resolve against the viewport; a popup must not be cut off by its page's
  `scrollView`. For Popover, `width` affects the trigger/layout and
  `contentWidth` affects the popup; do not make a wide menu stretch its
  trigger. The default popover leaves focus on the trigger and is nonmodal;
  opt into `initialFocus="first"` when the first control should receive focus.
  Modal dialogs need a separate modal containment contract.
  `allowOutsideWindow` requests a native popup, so verify the runtime feature
  and platform backend; unsupported hosts fall back inside the window. On the
  current Linux backend, native popups use X11. Check the alpha, shadow, and
  zoom details in [native popup surfaces](references/argui-native-popup-surfaces.md).
  For screen overlays and native pointer holes, read
  [desktop overlay pitfalls](references/argui-desktop-overlays.md).
  Keep new built-in property IDs distinct from all existing IDs, including IDs
  declared later in `builtin.rs`.
- Icons belong to the application. A widget may accept app-provided JSX or
  media slots, but must not require Tabler or another icon set. An icon-only
  action still needs an accessible name. The gallery can choose its own icons
  without turning that choice into a library dependency. A release includes
  SVGs discovered from reachable static TSX references; never maintain a
  separate manual release icon list. Resolve dynamic asset choices explicitly
  or let the build report that they cannot be traced. Align SVG and text by
  their layout boxes with `row alignItems="center"`. In an InputField, match
  the `textInput` height to its 20 px line height; a taller editor box makes
  the glyph appear off-center next to the SVG even when the row is centered.
  SVG artwork with asymmetric whitespace inside its own viewBox may still need
  an asset-level viewBox fix; do not encode a Tabler-specific widget offset.
- Keep navigation labels on one left edge. Use `Button contentAlign="start"`
  for full-width entries and a matching inset for category headings. A
  category is a heading, not an extra page; keep widget pages and example
  pages separately discoverable in the sidebar.
- Compose adjacent actions with `ButtonGroup`, not hand-painted borders on
  each button. Label the native group for assistive technology; each button
  remains separately keyboard-reachable. Use `ButtonGroupSeparator` for visible
  divisions, `ButtonGroupText` for static content, and `directionScope="rtl"`
  when the group follows right-to-left content. Keep its default width intrinsic
  in a column; use `width` or `alignSelf="stretch"` only when the group should
  fill the page. A chosen option in an action
  group should expose `pressed` as well as its selected visual paint.
  See [ButtonGroup and Select layout and motion](references/argui-button-group-select.md)
  for dividers that follow mixed control heights.
- A Neutral ghost button hovers with `muted`; place it on `background` or
  `sidebar` so the state does not disappear into an identical muted parent.
  For a sidebar, scope `ghostHover` to `sidebarAccent` and color the active
  label with `sidebarPrimary` while keeping the reusable Button icon agnostic.
  `pressed` gives toggle-style ghost buttons
  selected foreground and an active surface; color app-provided SVG children
  from the same state. Shadcn-style Button variants are `default`, `outline`,
  `secondary`, `ghost`, `destructive`, and `link`; sizes include the standard
  text and icon sizes. Press motion is native and `pressAnimation={false}`
  disables it. Keep hover paint and keyboard focus border on the same rounded
  rectangle. For grouped controls, clip at the outer `ButtonGroup` radius and
  let its focus border follow that radius; the joined children have square
  internal edges. This prevents square focus or hover corners escaping the
  parent frame.
- In `Select variant="shadcn"`, the placeholder is an option too. It must
  respond to pointer hover even after another value is selected; use native
  `hoverBackground` on its row, as on every other option.
  The [Select motion notes](references/argui-button-group-select.md) cover the
  retained chevron transition.
- Put layout demonstrations in an `Examples / Layouting` view. Explain the
  property beside an observable scene: resize for `%` versus `grow`, constrain
  a flex item with long text for overflow, and scroll real excess content.
  Put explanatory text inside a visible rectangle so its wrapping and clipping
  can be observed directly. `Examples / Animation` should demonstrate native
  motion with text inside the moving surface and a pause control. Prefer native
  hover, transitions and loops over a JavaScript timer that rebuilds the tree
  every frame. Do not spread anonymous test boxes through widget pages.
- Before designing motion or diagnosing animation stutter, read
  [the animation architecture guide](references/animation-architecture.md).
  For gradients, status indicators, voice surfaces, and throbbers, also read
  [the expressive motion guide](references/expressive-motion.md).
  Choose the smallest native invalidation path that can express the effect,
  design the idle and active states deliberately, and measure the presented
  result rather than assuming every animation is a compositor update.
- Size Select's popup from its actual rows and header, up to a viewport cap.
  A short list should show its final option without requiring a tiny extra
  scroll. For `variant="shadcn"`, keep the field's visible value and the
  popup's group label in sync with selection; preserve keyboard focus,
  accessible names, disabled options, and controlled-value semantics.
- Structured `border`, `shadow`, `radii`, `transform`, grid tracks and
  container rules must pass the generated types and native wire validation.
  For a custom native primitive, export its complete `SchemaRegistry` contract
  and run `argui-generate-jsx` for both adapters.

For gallery checks, use `bun run check:ts` and `bun run test:ts`; the latter
selects Solid's browser condition and ignores `OLD_API/`. Follow the graphical
testing procedure in `AGENTS.md`, or the user's explicit request to inspect the
visible app themselves. The final `quality.sh` gate is governed by `AGENTS.md`.
