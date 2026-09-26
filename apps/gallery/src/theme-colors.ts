import type { ThemeRuntime } from '@argui/host'
import type { WidgetTheme } from '@argui/widgets/solid'

// Tailwind v4 color-700 and color-400 values. Neutral keeps shadcn's own tokens.
export const tailwindFamilies = {
  Slate: ['oklch(0.372 0.044 257.287)', 'oklch(0.704 0.04 256.788)', 257],
  Gray: ['oklch(0.373 0.034 259.733)', 'oklch(0.707 0.022 261.325)', 260],
  Zinc: ['oklch(0.37 0.013 285.805)', 'oklch(0.705 0.015 286.067)', 286],
  Stone: ['oklch(0.374 0.01 67.558)', 'oklch(0.709 0.01 56.259)', 56],
  Mauve: ['oklch(0.364 0.029 323.89)', 'oklch(0.711 0.019 323.02)', 323],
  Olive: ['oklch(0.394 0.023 107.4)', 'oklch(0.737 0.021 106.9)', 107],
  Mist: ['oklch(0.378 0.015 216)', 'oklch(0.723 0.014 214.4)', 215],
  Taupe: ['oklch(0.367 0.016 35.7)', 'oklch(0.714 0.014 41.2)', 40],
  Red: ['oklch(0.505 0.213 27.518)', 'oklch(0.704 0.191 22.216)', 25],
  Orange: ['oklch(0.553 0.195 38.402)', 'oklch(0.75 0.183 55.934)', 48],
  Amber: ['oklch(0.555 0.163 48.998)', 'oklch(0.828 0.189 84.429)', 70],
  Yellow: ['oklch(0.554 0.135 66.442)', 'oklch(0.852 0.199 91.936)', 90],
  Lime: ['oklch(0.532 0.157 131.589)', 'oklch(0.841 0.238 128.85)', 130],
  Green: ['oklch(0.527 0.154 150.069)', 'oklch(0.792 0.209 151.711)', 151],
  Emerald: ['oklch(0.508 0.118 165.612)', 'oklch(0.765 0.177 163.223)', 164],
  Teal: ['oklch(0.511 0.096 186.391)', 'oklch(0.777 0.152 181.912)', 183],
  Cyan: ['oklch(0.52 0.105 223.128)', 'oklch(0.789 0.154 211.53)', 216],
  Sky: ['oklch(0.5 0.134 242.749)', 'oklch(0.746 0.16 232.661)', 238],
  Blue: ['oklch(0.488 0.243 264.376)', 'oklch(0.707 0.165 254.624)', 260],
  Indigo: ['oklch(0.457 0.24 277.023)', 'oklch(0.673 0.182 276.935)', 277],
  Violet: ['oklch(0.491 0.27 292.581)', 'oklch(0.702 0.183 293.541)', 293],
  Purple: ['oklch(0.496 0.265 301.924)', 'oklch(0.714 0.203 305.504)', 303],
  Fuchsia: ['oklch(0.518 0.253 323.949)', 'oklch(0.74 0.238 322.16)', 323],
  Pink: ['oklch(0.525 0.223 3.958)', 'oklch(0.718 0.202 349.761)', 350],
  Rose: ['oklch(0.514 0.222 16.935)', 'oklch(0.712 0.194 13.428)', 15],
} as const

/** Color family choices offered by both gallery adapters. */
export type ColorFamily = 'Neutral' | keyof typeof tailwindFamilies

/** All selectable color families, including the unmodified shadcn Neutral theme. */
export const colorFamilies: ColorFamily[] = ['Neutral', ...Object.keys(tailwindFamilies) as (keyof typeof tailwindFamilies)[]]

/** Returns the primary swatch for a family in the current appearance. */
export function familySwatch(family: ColorFamily, dark: boolean): string {
  if (family === 'Neutral') return dark ? 'oklch(0.922 0 0)' : 'oklch(0.205 0 0)'
  return tailwindFamilies[family][dark ? 1 : 0]
}

/** Resolves a Tailwind family's semantic color tokens with readable light and dark foreground pairs. */
export function familyOverrides(family: ColorFamily, dark: boolean): Partial<WidgetTheme> {
  if (family === 'Neutral') return {}
  const [lightPrimary, darkPrimary, hue] = tailwindFamilies[family]
  const primary = dark ? darkPrimary : lightPrimary
  const primaryForeground = dark ? 'oklch(0.145 0 0)' : 'oklch(0.985 0 0)'
  const quiet = ['Slate', 'Gray', 'Zinc', 'Stone', 'Mauve', 'Olive', 'Mist', 'Taupe'].includes(family)
  const muted = dark ? `oklch(0.269 ${quiet ? 0.008 : 0.025} ${hue})` : `oklch(0.97 ${quiet ? 0.005 : 0.018} ${hue})`
  const border = dark ? `oklch(0.38 ${quiet ? 0.012 : 0.03} ${hue})` : `oklch(0.91 ${quiet ? 0.007 : 0.025} ${hue})`
  const background = dark ? `oklch(0.145 ${quiet ? 0.004 : 0.01} ${hue})` : `oklch(0.995 ${quiet ? 0.002 : 0.006} ${hue})`
  const card = dark ? `oklch(0.205 ${quiet ? 0.007 : 0.018} ${hue})` : 'oklch(1 0 0)'
  const ring = dark ? darkPrimary : lightPrimary
  return {
    background, surface: background, card, popover: card,
    sidebar: dark ? card : `oklch(0.985 ${quiet ? 0.004 : 0.01} ${hue})`,
    primary, primaryForeground, primaryHover: primary.replace(')', ' / 80%)'),
    sidebarPrimary: primary, sidebarPrimaryForeground: primaryForeground,
    ring, focusRing: ring, sidebarRing: ring,
    secondary: muted, muted, accent: muted, sidebarAccent: muted,
    surfaceHover: muted, controlHover: muted, ghostHover: dark ? muted.replace(')', ' / 50%)') : muted,
    outlineSurface: dark ? `oklch(0.32 0.025 ${hue})` : background,
    outlineBorder: border, outlineHover: dark ? `oklch(0.39 0.03 ${hue})` : muted,
    secondaryHover: dark ? `oklch(0.31 0.03 ${hue})` : `oklch(0.94 0.025 ${hue})`,
    border, input: border, sidebarBorder: border,
    chart1: lightPrimary, chart2: darkPrimary,
    chart3: `oklch(0.62 0.12 ${hue})`, chart4: `oklch(0.75 0.1 ${hue})`,
    chart5: `oklch(0.42 0.08 ${hue})`,
  }
}

const colorKeys = Object.keys(familyOverrides('Blue', false)) as (keyof WidgetTheme & string)[]

/** Atomically applies a color family and appearance to the shared native theme. */
export function applyGalleryTheme(runtime: ThemeRuntime<WidgetTheme>, family: ColorFamily, appearance: 'light' | 'dark' | 'system'): void {
  const dark = appearance === 'dark' || (appearance === 'system' && runtime.snapshot().systemScheme === 'dark')
  runtime.update(family === 'Neutral'
    ? { variant: appearance, removeOverrides: colorKeys }
    : { variant: appearance, overrides: familyOverrides(family, dark) })
}

/** Keeps a selected color family in sync when the system appearance changes. */
export function watchSystemFamily(runtime: ThemeRuntime<WidgetTheme>, family: () => ColorFamily): () => void {
  let scheme = runtime.snapshot().systemScheme
  return runtime.subscribe((snapshot) => {
    if (snapshot.systemScheme === scheme) return
    scheme = snapshot.systemScheme
    if (snapshot.variant === 'system' && family() !== 'Neutral') {
      runtime.update({ overrides: familyOverrides(family(), scheme === 'dark') })
    }
  })
}
