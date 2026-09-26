import type { ThemeDefinition } from '@argui/host'

export type Accent = 'blue' | 'violet' | 'emerald'
export type ThemeMode = 'light' | 'dark' | 'system'

/** Resolved colors shared by Argui widgets and their host application. */
export type Palette = {
  background: string
  surface: string
  surfaceRaised: string
  overlaySurface: string
  overlayShadow: string
  border: string
  foreground: string
  muted: string
  accent: string
  accentHover: string
  accentPressed: string
  accentText: string
  surfaceHover: string
  surfacePressed: string
  destructive: string
  destructiveHover: string
  destructivePressed: string
  controlRadius: number
  controlPadding: number
  controlFontSize: number
  inputHeight: number
  overlayRadius: number
  overlayPadding: number
  overlayShadowBlur: number
  dialogRadius: number
  dialogPadding: number
  dialogShadowBlur: number
}

/** Accent overrides applied as one theme transaction when the user changes color. */
export const accentOverrides: Record<Accent, Pick<Palette, 'accent' | 'accentHover' | 'accentPressed'>> = {
  blue: { accent: '#2563eb', accentHover: '#1d4ed8', accentPressed: '#1e40af' },
  violet: { accent: '#7c3aed', accentHover: '#6d28d9', accentPressed: '#5b21b6' },
  emerald: { accent: '#059669', accentHover: '#047857', accentPressed: '#065f46' },
}

const metrics = {
  controlRadius: 9, controlPadding: 10, controlFontSize: 14, inputHeight: 42,
  overlayRadius: 10, overlayPadding: 16, overlayShadowBlur: 14,
  dialogRadius: 16, dialogPadding: 24, dialogShadowBlur: 24,
}

const light: Palette = {
  background: '#f5f7fb', surface: '#ffffff', surfaceRaised: '#edf0f7',
  overlaySurface: '#ffffffc0', overlayShadow: '#17243b38',
  border: '#d8deea', foreground: '#171a24', muted: '#596377',
  ...accentOverrides.blue, accentText: '#ffffff',
  surfaceHover: '#e7ebf4', surfacePressed: '#dce3f0',
  destructive: '#dc2626', destructiveHover: '#b91c1c', destructivePressed: '#991b1b',
  ...metrics,
}

const dark: Palette = {
  background: '#101116', surface: '#1a1b23', surfaceRaised: '#252735',
  overlaySurface: '#1a1b23d0', overlayShadow: '#00000066',
  border: '#383b4b', foreground: '#f5f6fa', muted: '#a3a8b9',
  ...accentOverrides.blue, accentText: '#ffffff',
  surfaceHover: '#303345', surfacePressed: '#3a3e52',
  destructive: '#dc2626', destructiveHover: '#b91c1c', destructivePressed: '#991b1b',
  ...metrics,
}

/** Typed token definitions for the built-in widget colors. */
export const widgetThemeDefinition: ThemeDefinition<Palette> = {
  tokens: Object.fromEntries(Object.entries(light).map(([key, value]) =>
    [key, { type: typeof value === 'number' ? key === 'controlFontSize' ? 'FontSize' : 'Length' : 'Color',
      default: value,
      impact: ['controlPadding', 'controlFontSize', 'inputHeight', 'overlayPadding', 'dialogPadding'].includes(key)
        ? 'Layout' : 'Paint' }])) as ThemeDefinition<Palette>['tokens'],
  variants: { light, dark },
  initialVariant: 'system',
  systemVariants: { light: 'light', dark: 'dark' },
}

/** Gives a hex accent the same 38% alpha as native text selection. */
export function selectionTint(accent: string): string {
  if (/^#[0-9a-fA-F]{6}$/.test(accent)) return `${accent}61`
  if (/^#[0-9a-fA-F]{8}$/.test(accent)) return `${accent.slice(0, 7)}61`
  return '#337af561'
}
