import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { buttonPaint } from '../src/shared/button-paint'
import { keyName, nextEnabledOption, type SelectOption } from '../src/shared/types'
import { neutralDark, neutralLight, widgetThemeDefinition, type WidgetTheme } from '../src/shared/theme'

const theme = widgetThemeDefinition.variants!.light as WidgetTheme

test('button variants use official Neutral semantic colors and native state colors', () => {
  expect(buttonPaint('default', theme)).toMatchObject({
    background: theme.primary, foreground: theme.primaryForeground, hover: theme.primaryHover,
  })
  expect(buttonPaint('outline', theme)).toMatchObject({ background: theme.background, hover: theme.muted })
  expect(buttonPaint('secondary', theme)).toMatchObject({ background: theme.secondary, hover: theme.secondaryHover })
  expect(buttonPaint('ghost', theme).background).toBe('transparent')
  expect(buttonPaint('ghost', theme, true)).toMatchObject({ background: theme.accent, foreground: theme.primary })
  expect(buttonPaint('ghost', theme).hover).toBe(theme.ghostHover)
  expect(buttonPaint('destructive', theme)).toMatchObject({ background: theme.destructiveSurface, foreground: theme.destructive })
  expect(buttonPaint('link', theme)).toMatchObject({ background: 'transparent', foreground: theme.primary })
  expect(buttonPaint('default', theme).pressed).toBe(buttonPaint('default', theme).hover)
})

test('select keyboard navigation wraps and skips disabled options', () => {
  const options: SelectOption[] = [
    { value: 'a', label: 'A' },
    { value: 'b', label: 'B', disabled: true },
    { value: 'c', label: 'C' },
  ]
  expect(nextEnabledOption(options, -1, 1)).toBe(0)
  expect(nextEnabledOption(options, 0, 1)).toBe(2)
  expect(nextEnabledOption(options, 2, 1)).toBe(0)
  expect(nextEnabledOption([{ value: 'x', label: 'X', disabled: true }], -1, 1)).toBe(-1)
  expect(nextEnabledOption([], -1, 1)).toBe(-1)
})

test('select keys accept the native payload shape and ignore malformed data', () => {
  expect(keyName('ArrowDown')).toBe('ArrowDown')
  expect(keyName({ key: 'Enter' })).toBe('Enter')
  expect(keyName({ key: 'Enter', state: 'released' })).toBeUndefined()
  expect(keyName({ key: 'Enter', state: 'pressed' })).toBe('Enter')
  expect(keyName({ key: 2 })).toBeUndefined()
  expect(keyName(null)).toBeUndefined()
})

test('widget theme exposes every official Neutral role and system variants', () => {
  const reference = readFileSync(new URL('../../../docs/references/shadcn-ui/cli/index.css', import.meta.url), 'utf8')
  const cssBlock = (selector: string) => reference.match(new RegExp(`${selector} \\{([^}]+)\\}`))?.[1] ?? ''
  const cssValue = (block: string, name: string) => {
    const cssName = name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`).replace(/(\D)(\d)/g, '$1-$2')
    return block.match(new RegExp(`--${cssName}: ([^;]+);`))?.[1]
  }
  const lightCss = cssBlock(':root')
  const darkCss = cssBlock('\\.dark')
  for (const name of Object.keys(neutralLight) as (keyof typeof neutralLight)[]) {
    expect(neutralLight[name]).toBe(cssValue(lightCss, name) ?? '')
    expect(neutralDark[name]).toBe(cssValue(darkCss, name) ?? '')
    expect(widgetThemeDefinition.tokens[name]?.default).toBe(neutralLight[name])
    expect(widgetThemeDefinition.variants?.dark?.[name]).toBe(neutralDark[name])
  }
  expect(widgetThemeDefinition.systemVariants).toEqual({ light: 'light', dark: 'dark' })
  expect(widgetThemeDefinition.initialVariant).toBeUndefined()
})

test('widget aliases keep text and surfaces aligned with semantic Neutral roles', () => {
  for (const variant of Object.values(widgetThemeDefinition.variants!) as WidgetTheme[]) {
    expect(variant.surface).toBe(variant.background)
    expect(variant.text).toBe(variant.foreground)
    expect(variant.textMuted).toBe(variant.mutedForeground)
    expect(variant.focusRing).toBe(variant.ring)
    expect(variant.danger).toBe(variant.destructive)
  }
})
