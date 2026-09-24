import { describe, expect, test } from 'bun:test'
import { numberVariable, parseThemeVariables, stringVariable, themeJson,
  themePresets, validateExampleTheme } from '../src/theme-variables'

describe('application theme variables', () => {
  test('round-trips extra names without a framework token list', () => {
    const custom = { ...themePresets.Aurora, cardCorner: 13, navLabel: '#123456' }
    const restored = parseThemeVariables(themeJson(custom))
    validateExampleTheme(restored)
    expect(restored).toEqual(custom)
    expect(numberVariable(restored, 'cardCorner')).toBe(13)
    expect(stringVariable(restored, 'navLabel')).toBe('#123456')
  })

  test('rejects invalid JSON values and incompatible property bindings', () => {
    expect(() => parseThemeVariables('{"nested":{"color":"#123456"}}')).toThrow('nested')
    expect(() => numberVariable({ spacing: '#abcdef' }, 'spacing')).toThrow('spacing')
    expect(() => validateExampleTheme({ ...themePresets.Aurora, radius: -1 })).toThrow('radius')
    expect(() => validateExampleTheme({ ...themePresets.Aurora, accent: 'not a color' })).toThrow('accent')
  })
})
