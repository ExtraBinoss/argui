# Widget theme colors

`widgetThemeDefinition` provides the official shadcn/ui Neutral light and dark
semantic colors. The values in `packages/widgets/src/shared/theme.ts` match the
local [shadcn Neutral CSS reference](../references/shadcn-ui/cli/index.css):
`background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`,
`accent`, `destructive`, `border`, `input`, `ring`, five chart colors, and the
sidebar roles. They keep their CSS meanings. In particular, `accent` is the
subtle hover surface; primary actions use `primary` and `primaryForeground`.

The theme starts in System mode. `runtime.update({ variant: 'light' })` and
`runtime.update({ variant: 'dark' })` select a fixed variant;
`runtime.update({ variant: 'system' })` resumes the OS preference. A control
that indicates the selected choice should read the theme snapshot's `variant`,
not `resolvedVariant`, because System can currently resolve to Light or Dark.

Native `Color` and solid `Brush` values accept hexadecimal `#RGB`, `#RGBA`,
`#RRGGBB`, `#RRGGBBAA`, CSS `oklch(L C H / alpha)`, `rgb(...)`, and `rgba(...)`.
RGB channels can use 0–255 numbers or percentages. Alpha can use a 0–1 number
or percentage. Both comma syntax (`rgba(255, 0, 0, 0.5)`) and modern space
syntax (`rgb(255 0 0 / 50%)`) work. `transparent` is also supported. Invalid
channels fail native validation. The theme bridge sends resolved colors back
as 8-bit sRGB hex; the authored Neutral tokens remain `oklch(...)` in the
definition.

Shadcn exposes its semantic colors as Tailwind utility colors through CSS
`@theme inline`. Argui exposes the semantic roles directly in TSX. A Tailwind
name such as `blue-500` is not a color literal; use its actual CSS value in a
theme override when a custom color is needed.

The Argui-only overlay tokens (`overlaySurface`, `overlayBlur`, radius,
padding, and shadow) serve floating components while preserving the exact
Neutral semantic colors. `overlaySurface` is intentionally translucent for a
blurred Popover; `popover` itself retains the official opaque value.
