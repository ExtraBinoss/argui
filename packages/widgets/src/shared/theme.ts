import type { ThemeDefinition, ThemeValues } from '@argui/host'

/** Official shadcn/ui Neutral semantic color names, written in camelCase for TSX. */
export interface NeutralColors {
  background: string
  foreground: string
  card: string
  cardForeground: string
  popover: string
  popoverForeground: string
  primary: string
  primaryForeground: string
  secondary: string
  secondaryForeground: string
  muted: string
  mutedForeground: string
  accent: string
  accentForeground: string
  destructive: string
  border: string
  input: string
  ring: string
  chart1: string
  chart2: string
  chart3: string
  chart4: string
  chart5: string
  sidebar: string
  sidebarForeground: string
  sidebarPrimary: string
  sidebarPrimaryForeground: string
  sidebarAccent: string
  sidebarAccentForeground: string
  sidebarBorder: string
  sidebarRing: string
}

/** Theme tokens shared by widgets in both framework adapters. */
export interface WidgetTheme extends ThemeValues, NeutralColors {
  surface: string
  surfaceHover: string
  controlHover: string
  text: string
  textMuted: string
  focusRing: string
  danger: string
  primaryHover: string
  secondaryHover: string
  ghostHover: string
  outlineSurface: string
  outlineBorder: string
  outlineHover: string
  destructiveSurface: string
  destructiveHover: string
  radius: number
  spacing: number
  overlaySurface: string
  overlayBlur: number
  overlayRadius: number
  overlayPadding: number
  overlayWidth: number
  overlayBorderWidth: number
  overlayShadowColor: string
  overlayShadowBlur: number
  overlayShadowOffsetY: number
}

/** Official shadcn/ui Neutral light semantic values. */
export const neutralLight: NeutralColors = {
  background: 'oklch(1 0 0)', foreground: 'oklch(0.145 0 0)',
  card: 'oklch(1 0 0)', cardForeground: 'oklch(0.145 0 0)',
  popover: 'oklch(1 0 0)', popoverForeground: 'oklch(0.145 0 0)',
  primary: 'oklch(0.205 0 0)', primaryForeground: 'oklch(0.985 0 0)',
  secondary: 'oklch(0.97 0 0)', secondaryForeground: 'oklch(0.205 0 0)',
  muted: 'oklch(0.97 0 0)', mutedForeground: 'oklch(0.556 0 0)',
  accent: 'oklch(0.97 0 0)', accentForeground: 'oklch(0.205 0 0)',
  destructive: 'oklch(0.577 0.245 27.325)',
  border: 'oklch(0.922 0 0)', input: 'oklch(0.922 0 0)', ring: 'oklch(0.708 0 0)',
  chart1: 'oklch(0.87 0 0)', chart2: 'oklch(0.556 0 0)',
  chart3: 'oklch(0.439 0 0)', chart4: 'oklch(0.371 0 0)',
  chart5: 'oklch(0.269 0 0)',
  sidebar: 'oklch(0.985 0 0)', sidebarForeground: 'oklch(0.145 0 0)',
  sidebarPrimary: 'oklch(0.205 0 0)', sidebarPrimaryForeground: 'oklch(0.985 0 0)',
  sidebarAccent: 'oklch(0.97 0 0)', sidebarAccentForeground: 'oklch(0.205 0 0)',
  sidebarBorder: 'oklch(0.922 0 0)', sidebarRing: 'oklch(0.708 0 0)',
}

/** Official shadcn/ui Neutral dark semantic values. */
export const neutralDark: NeutralColors = {
  background: 'oklch(0.145 0 0)', foreground: 'oklch(0.985 0 0)',
  card: 'oklch(0.205 0 0)', cardForeground: 'oklch(0.985 0 0)',
  popover: 'oklch(0.205 0 0)', popoverForeground: 'oklch(0.985 0 0)',
  primary: 'oklch(0.922 0 0)', primaryForeground: 'oklch(0.205 0 0)',
  secondary: 'oklch(0.269 0 0)', secondaryForeground: 'oklch(0.985 0 0)',
  muted: 'oklch(0.269 0 0)', mutedForeground: 'oklch(0.708 0 0)',
  accent: 'oklch(0.269 0 0)', accentForeground: 'oklch(0.985 0 0)',
  destructive: 'oklch(0.704 0.191 22.216)',
  border: 'oklch(1 0 0 / 10%)', input: 'oklch(1 0 0 / 15%)', ring: 'oklch(0.556 0 0)',
  chart1: 'oklch(0.87 0 0)', chart2: 'oklch(0.556 0 0)',
  chart3: 'oklch(0.439 0 0)', chart4: 'oklch(0.371 0 0)',
  chart5: 'oklch(0.269 0 0)',
  sidebar: 'oklch(0.205 0 0)', sidebarForeground: 'oklch(0.985 0 0)',
  sidebarPrimary: 'oklch(0.488 0.243 264.376)', sidebarPrimaryForeground: 'oklch(0.985 0 0)',
  sidebarAccent: 'oklch(0.269 0 0)', sidebarAccentForeground: 'oklch(0.985 0 0)',
  sidebarBorder: 'oklch(1 0 0 / 10%)', sidebarRing: 'oklch(0.556 0 0)',
}

const light: WidgetTheme = {
  ...neutralLight,
  surface: neutralLight.background, surfaceHover: neutralLight.muted,
  controlHover: neutralLight.muted, text: neutralLight.foreground,
  textMuted: neutralLight.mutedForeground, focusRing: neutralLight.ring,
  danger: neutralLight.destructive,
  primaryHover: 'oklch(0.205 0 0 / 80%)',
  secondaryHover: 'oklch(0.92875 0 0)',
  ghostHover: neutralLight.muted,
  outlineSurface: neutralLight.background, outlineBorder: neutralLight.border,
  outlineHover: neutralLight.muted,
  destructiveSurface: 'oklch(0.577 0.245 27.325 / 10%)',
  destructiveHover: 'oklch(0.577 0.245 27.325 / 20%)',
  radius: 10, spacing: 8,
  overlaySurface: 'oklch(1 0 0 / 92%)', overlayBlur: 10,
  overlayRadius: 10, overlayPadding: 16, overlayWidth: 280,
  overlayBorderWidth: 1, overlayShadowColor: 'oklch(0 0 0 / 12%)',
  overlayShadowBlur: 14, overlayShadowOffsetY: 4,
}

const dark: WidgetTheme = {
  ...neutralDark,
  surface: neutralDark.background, surfaceHover: neutralDark.muted,
  controlHover: neutralDark.muted, text: neutralDark.foreground,
  textMuted: neutralDark.mutedForeground, focusRing: neutralDark.ring,
  danger: neutralDark.destructive,
  primaryHover: 'oklch(0.922 0 0 / 80%)',
  secondaryHover: 'oklch(0.3048 0 0)',
  ghostHover: 'oklch(0.269 0 0 / 50%)',
  outlineSurface: 'oklch(1 0 0 / 30%)', outlineBorder: neutralDark.input,
  outlineHover: 'oklch(1 0 0 / 50%)',
  destructiveSurface: 'oklch(0.704 0.191 22.216 / 20%)',
  destructiveHover: 'oklch(0.704 0.191 22.216 / 30%)',
  radius: 10, spacing: 8,
  overlaySurface: 'oklch(0.205 0 0 / 92%)', overlayBlur: 10,
  overlayRadius: 10, overlayPadding: 16, overlayWidth: 280,
  overlayBorderWidth: 1, overlayShadowColor: 'oklch(0 0 0 / 50%)',
  overlayShadowBlur: 14, overlayShadowOffsetY: 4,
}

const layoutTokens = new Set(['spacing', 'overlayPadding', 'overlayWidth'])

/** Default system-aware Neutral theme, with reusable Argui overlay tokens. */
export const widgetThemeDefinition: ThemeDefinition<WidgetTheme> = {
  tokens: Object.fromEntries(Object.entries(light).map(([name, value]) => [name, {
    type: typeof value === 'number' ? 'Length' : 'Color',
    default: value,
    impact: layoutTokens.has(name) ? 'Layout' : 'Paint',
  }])) as ThemeDefinition<WidgetTheme>['tokens'],
  variants: { light, dark },
  systemVariants: { light: 'light', dark: 'dark' },
}
