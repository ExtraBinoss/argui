export type Accent = 'blue' | 'violet' | 'emerald'
export type ThemeMode = 'light' | 'dark'

export interface Palette {
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
}

const accents: Record<Accent, readonly [string, string, string]> = {
  blue: ['#2563eb', '#1d4ed8', '#1e40af'],
  violet: ['#7c3aed', '#6d28d9', '#5b21b6'],
  emerald: ['#059669', '#047857', '#065f46'],
}

/** Resolves the gallery's shared native color tokens. */
export function palette(mode: ThemeMode, accent: Accent): Palette {
  return mode === 'dark'
    ? {
        background: '#101116', surface: '#1a1b23', surfaceRaised: '#252735',
        overlaySurface: '#1a1b23d0', overlayShadow: '#00000066',
        border: '#383b4b', foreground: '#f5f6fa', muted: '#a3a8b9',
        accent: accents[accent][0], accentHover: accents[accent][1],
        accentPressed: accents[accent][2], accentText: '#ffffff',
        surfaceHover: '#303345', surfacePressed: '#3a3e52',
        destructive: '#dc2626', destructiveHover: '#b91c1c', destructivePressed: '#991b1b',
      }
    : {
        background: '#f5f7fb', surface: '#ffffff', surfaceRaised: '#edf0f7',
        overlaySurface: '#ffffffc0', overlayShadow: '#17243b38',
        border: '#d8deea', foreground: '#171a24', muted: '#596377',
        accent: accents[accent][0], accentHover: accents[accent][1],
        accentPressed: accents[accent][2], accentText: '#ffffff',
        surfaceHover: '#e7ebf4', surfacePressed: '#dce3f0',
        destructive: '#dc2626', destructiveHover: '#b91c1c', destructivePressed: '#991b1b',
      }
}
