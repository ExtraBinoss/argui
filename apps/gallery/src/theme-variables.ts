/** A theme is an application-owned map, with no reserved variable names. */
export type ThemeVariables = Record<string, string | number | boolean>

/** Three example themes whose values are ordinary TSX property inputs. */
export const themePresets = {
  Aurora: {
    canvas: '#101827', panel: '#26344b99', ink: '#f4f8ff', muted: '#dce8f6',
    accent: '#7be5cb', radius: 24, padding: 24, blur: 10, fontSize: 23,
    shadowBlur: 22, shadowColor: '#00000088',
  },
  Paper: {
    canvas: '#eee9df', panel: '#fffdf899', ink: '#282e35', muted: '#394653',
    accent: '#ce6754', radius: 4, padding: 18, blur: 0, fontSize: 21,
    shadowBlur: 5, shadowColor: '#282e3544',
  },
  Midnight: {
    canvas: '#070912', panel: '#171d3499', ink: '#f6f0ff', muted: '#d4ccea',
    accent: '#c69bff', radius: 38, padding: 30, blur: 20, fontSize: 25,
    shadowBlur: 30, shadowColor: '#000000aa',
  },
} as const satisfies Record<string, ThemeVariables>

/** Parses app-supplied JSON without restricting variable names. */
export function parseThemeVariables(json: string): ThemeVariables {
  const parsed: unknown = JSON.parse(json)
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('Theme JSON must contain an object of variables.')
  }
  const variables: ThemeVariables = {}
  for (const [name, value] of Object.entries(parsed)) {
    if (!name.trim()) throw new Error('Theme variable names cannot be empty.')
    if (typeof value !== 'string' && typeof value !== 'boolean'
      && !(typeof value === 'number' && Number.isFinite(value))) {
      throw new Error(`Variable "${name}" must be a string, finite number, or boolean.`)
    }
    variables[name] = value
  }
  return variables
}

/** Reads a string variable for a string or color TSX property. */
export function stringVariable(variables: ThemeVariables, name: string): string {
  const value = variables[name]
  if (typeof value !== 'string') throw new Error(`Variable "${name}" must be a string.`)
  return value
}

/** Reads a finite numeric variable for a numeric TSX property. */
export function numberVariable(variables: ThemeVariables, name: string): number {
  const value = variables[name]
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new Error(`Variable "${name}" must be a finite number.`)
  }
  return value
}

/** Checks only the properties used by this example before applying its JSON. */
export function validateExampleTheme(variables: ThemeVariables): void {
  for (const name of ['canvas', 'panel', 'ink', 'muted', 'accent', 'shadowColor']) {
    if (!/^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(stringVariable(variables, name))) {
      throw new Error(`Variable "${name}" must be a six or eight digit hex color.`)
    }
  }
  for (const [name, min, max] of [
    ['radius', 0, 64], ['padding', 0, 48], ['blur', 0, 40],
    ['fontSize', 12, 36], ['shadowBlur', 0, 48],
  ] as const) {
    const value = numberVariable(variables, name)
    if (value < min || value > max) throw new Error(`Variable "${name}" must be between ${min} and ${max}.`)
  }
}

/** Serializes the active variables for storage by the surrounding application. */
export function themeJson(variables: ThemeVariables): string {
  return JSON.stringify(variables, null, 2)
}
