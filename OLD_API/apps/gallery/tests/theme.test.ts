import { describe, expect, test } from 'bun:test'
import { parseThemeVariables, previewTokens, themeJson, themePresets,
  galleryThemeDefinition, type GalleryTokens } from '../src/theme'

describe('gallery theme tokens', () => {
  test('presets round-trip through typed app-owned token JSON', () => {
    const restored = parseThemeVariables(themeJson(themePresets.Aurora))
    expect(restored).toEqual(themePresets.Aurora)
    const defaults = Object.fromEntries(Object.entries(galleryThemeDefinition.tokens)
      .map(([key, definition]) => [key, definition.default])) as GalleryTokens
    expect(previewTokens(defaults)).toEqual(restored)
    expect(galleryThemeDefinition.tokens.samplePadding.impact).toBe('Layout')
  })

  test('rejects unknown and incorrectly typed override values', () => {
    expect(() => parseThemeVariables('{"nested":{"color":"#123456"}}')).toThrow('nested')
    expect(() => parseThemeVariables(themeJson({ ...themePresets.Aurora, sampleRadius: -1 }))).toThrow('sampleRadius')
    expect(() => parseThemeVariables(themeJson({ ...themePresets.Aurora, sampleAccent: 'not a color' }))).toThrow('sampleAccent')
  })
})
