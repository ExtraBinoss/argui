import type { ThemeDefinition } from '@argui/host'
import { widgetThemeDefinition, type Palette } from '@argui/widgets/theme'

/** Extra typed tokens used by the live theme editor in this application. */
export type PreviewTokens = {
  sampleCanvas: string
  samplePanel: string
  sampleInk: string
  sampleMuted: string
  sampleAccent: string
  sampleRadius: number
  samplePadding: number
  sampleBlur: number
  sampleFontSize: number
  sampleShadowBlur: number
  sampleShadowColor: string
}

export type GalleryTokens = Palette & PreviewTokens

/** Example presets are sparse overrides of the application's own typed tokens. */
export const themePresets = {
  Aurora: {
    sampleCanvas: '#101827', samplePanel: '#26344b99', sampleInk: '#f4f8ff',
    sampleMuted: '#dce8f6', sampleAccent: '#7be5cb', sampleRadius: 24,
    samplePadding: 24, sampleBlur: 10, sampleFontSize: 23,
    sampleShadowBlur: 22, sampleShadowColor: '#00000088',
  },
  Paper: {
    sampleCanvas: '#eee9df', samplePanel: '#fffdf899', sampleInk: '#282e35',
    sampleMuted: '#394653', sampleAccent: '#ce6754', sampleRadius: 4,
    samplePadding: 18, sampleBlur: 0, sampleFontSize: 21,
    sampleShadowBlur: 5, sampleShadowColor: '#282e3544',
  },
  Midnight: {
    sampleCanvas: '#070912', samplePanel: '#171d3499', sampleInk: '#f6f0ff',
    sampleMuted: '#d4ccea', sampleAccent: '#c69bff', sampleRadius: 38,
    samplePadding: 30, sampleBlur: 20, sampleFontSize: 25,
    sampleShadowBlur: 30, sampleShadowColor: '#000000aa',
  },
} as const satisfies Record<string, PreviewTokens>

const preview = themePresets.Aurora

/** One schema contains both widget colors and app-owned preview tokens. */
export const galleryThemeDefinition: ThemeDefinition<GalleryTokens> = {
  tokens: {
    ...widgetThemeDefinition.tokens,
    sampleCanvas: { type: 'Color', default: preview.sampleCanvas, impact: 'Paint' },
    samplePanel: { type: 'Color', default: preview.samplePanel, impact: 'Paint' },
    sampleInk: { type: 'Color', default: preview.sampleInk, impact: 'Paint' },
    sampleMuted: { type: 'Color', default: preview.sampleMuted, impact: 'Paint' },
    sampleAccent: { type: 'Color', default: preview.sampleAccent, impact: 'Paint' },
    sampleRadius: { type: 'Length', default: preview.sampleRadius, impact: 'Paint' },
    samplePadding: { type: 'Length', default: preview.samplePadding, impact: 'Layout' },
    sampleBlur: { type: 'Length', default: preview.sampleBlur, impact: 'Composite' },
    sampleFontSize: { type: 'FontSize', default: preview.sampleFontSize, impact: 'Layout' },
    sampleShadowBlur: { type: 'Length', default: preview.sampleShadowBlur, impact: 'Paint' },
    sampleShadowColor: { type: 'Color', default: preview.sampleShadowColor, impact: 'Paint' },
  },
  variants: widgetThemeDefinition.variants,
  initialVariant: 'system',
  systemVariants: { light: 'light', dark: 'dark' },
}

const color = /^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/
const ranges: Partial<Record<keyof PreviewTokens, readonly [number, number]>> = {
  sampleRadius: [0, 64], samplePadding: [0, 48], sampleBlur: [0, 40],
  sampleFontSize: [12, 36], sampleShadowBlur: [0, 48],
}

/** Parses and validates one complete set of app-owned preview overrides. */
export function parseThemeVariables(json: string): PreviewTokens {
  const parsed: unknown = JSON.parse(json)
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('Theme JSON must contain an object of variables.')
  }
  for (const [name, value] of Object.entries(parsed)) {
    if (!(name in preview)) throw new Error(`Unknown theme token "${name}".`)
    if (typeof preview[name as keyof PreviewTokens] === 'number') {
      const range = ranges[name as keyof PreviewTokens]!
      if (typeof value !== 'number' || !Number.isFinite(value) || value < range[0] || value > range[1]) {
        throw new Error(`Variable "${name}" must be between ${range[0]} and ${range[1]}.`)
      }
    } else if (typeof value !== 'string' || !color.test(value)) {
      throw new Error(`Variable "${name}" must be a six or eight digit hex color.`)
    }
  }
  for (const name of Object.keys(preview)) {
    if (!(name in parsed)) throw new Error(`Missing theme token "${name}".`)
  }
  return parsed as PreviewTokens
}

/** Selects preview tokens from a coherent app theme snapshot. */
export function previewTokens(values: GalleryTokens): PreviewTokens {
  return Object.fromEntries(Object.keys(preview).map((key) =>
    [key, values[key as keyof PreviewTokens]])) as PreviewTokens
}

/** Serializes preview tokens for editing or export. */
export function themeJson(variables: PreviewTokens): string {
  return JSON.stringify(variables, null, 2)
}
