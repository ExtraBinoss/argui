# Widget themes

`WidgetTheme` is the shared token contract used by the Solid and React widget
adapters. Create one host-owned `ThemeRuntime` for an application window, then
install the adapter's `ThemeProvider` above widgets. The runtime resolves
variants and overrides atomically; widgets read the resulting values through
`useTheme`. A local `ThemeScope` can override a few tokens for one subtree.

## Install the root theme

The gallery creates its runtime from the same public API:

```ts solid
import { createThemeRuntime, type NativeBridge } from '@argui/host'
import { widgetThemeDefinition, type WidgetTheme } from '@argui/widgets/solid'

export function createWidgetTheme(bridge: NativeBridge) {
  return createThemeRuntime<WidgetTheme>(bridge, widgetThemeDefinition)
}
```

Pass that runtime to `ThemeProvider` inside a mounted Solid tree. Dispose the
runtime when the window unmounts, after disposing its tree. For React, import
the same definition from `@argui/widgets/react` and `ThemeProvider` from
`@argui/react`.

```tsx solid
import { ThemeProvider, useTheme, useThemeSnapshot, ThemeScope } from '@argui/solid'
import type { ThemeRuntime } from '@argui/host'
import { Button, type WidgetTheme } from '@argui/widgets/solid'

function ThemeSettings(props: { runtime: ThemeRuntime<WidgetTheme> }) {
  const theme = useTheme<WidgetTheme>()
  const snapshot = useThemeSnapshot<WidgetTheme>()

  return <column gap={8} background={theme().background}>
    <text color={theme().foreground}
      text={`Selected: ${snapshot().variant ?? 'defaults'}`} />
    <row gap={8}>
      <Button onClick={() => props.runtime.update({ variant: 'system' })}>System</Button>
      <Button onClick={() => props.runtime.update({ variant: 'light' })}>Light</Button>
      <Button onClick={() => props.runtime.update({ variant: 'dark' })}>Dark</Button>
    </row>
    <ThemeScope<WidgetTheme> overrides={{ ghostHover: theme().sidebarAccent }}>
      <Button variant="ghost" onClick={() => props.runtime.update({ variant: 'system' })}>
        Follow system
      </Button>
    </ThemeScope>
  </column>
}

export function ThemedApp(props: { runtime: ThemeRuntime<WidgetTheme> }) {
  return <ThemeProvider runtime={props.runtime}>
    <ThemeSettings runtime={props.runtime} />
  </ThemeProvider>
}
```

`widgetThemeDefinition` defines light and dark variants and maps the system
appearance to them. It does not set `initialVariant`; the host's default is
`system`. `snapshot().variant` is the user's selection (`system`, `light`, or
`dark`); `snapshot().resolvedVariant` is the light or dark variant currently
used. A Settings selection indicator should therefore read `variant`.
Calling `runtime.update({ variant: 'system' })` restores system following.

Solid's `useTheme` and `useThemeSnapshot` return accessors, so read `theme()`
and `snapshot()`. React's versions return the values directly, so read
`theme.foreground` and `snapshot.variant`. Both hooks require a provider.
`ThemeScope` inherits the other tokens; an unknown token or a value with the
wrong type fails its local merge.

## Token roles

The Neutral palette uses semantic names: `background`/`foreground` for the
main surface and text, `card`, `popover`, `primary`, `secondary`, `muted`,
`accent`, `destructive`, `border`, `input`, `ring`, chart colors, and `sidebar*`
roles. `accent` is a subtle hover surface; primary actions use `primary` and
`primaryForeground`. The widget-specific tokens include `surface`, `text`,
`controlHover`, `focusRing`, `radius`, input and Select dimensions, and the
`overlay*` values used by floating surfaces. Their exact current values are
defined in [`packages/widgets/src/shared/theme.ts`](../../packages/widgets/src/shared/theme.ts).

Use semantic tokens for custom TSX paint:

```tsx solid
import { useTheme } from '@argui/solid'
import type { WidgetTheme } from '@argui/widgets/solid'

export function StatusLine() {
  const theme = useTheme<WidgetTheme>()
  return <rectangle padding={12} background={theme().card}>
    <text color={theme().cardForeground}>Ready</text>
  </rectangle>
}
```

Raw `<text>` does not automatically take the widget theme's foreground color;
bind its `color` explicitly. Check contrast in both variants, including hover,
focus, placeholder, and disabled states. Icons and other application media
can take the same tokens without making an icon library a widget dependency.

For a window-wide change, use an atomic runtime patch:

```ts
runtime.update({ overrides: { primary: '#2563eb' } })
runtime.update({ removeOverrides: ['primary'] })
```

The native color parser accepts hex `#RGB(A)` and `#RRGGBB(AA)`, `oklch(...)`,
`rgb(...)`, `rgba(...)`, and `transparent`. RGB channels and alpha are
validated. A Tailwind utility name such as `blue-500` is not a color literal;
pass its actual CSS color value. `ThemeRuntime.update` returns a coherent
snapshot and rejects updates after `dispose()`.
