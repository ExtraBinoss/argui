import type { WidgetTheme } from './theme'
import type { ButtonSize, ButtonVariant } from './types'

/** Resolves native button colors from shadcn Neutral roles without JavaScript hover state. */
export function buttonPaint(variant: ButtonVariant | undefined, theme: Readonly<WidgetTheme>, selected = false) {
  switch (variant ?? 'default') {
    case 'default':
      return { background: theme.primary, foreground: theme.primaryForeground,
        hover: theme.primaryHover }
    case 'outline':
      return { background: theme.outlineSurface, foreground: theme.foreground,
        hover: theme.outlineHover }
    case 'secondary':
      return { background: theme.secondary, foreground: theme.secondaryForeground,
        hover: theme.secondaryHover }
    case 'ghost':
      return { background: selected ? theme.accent : 'transparent',
        foreground: selected ? theme.primary : theme.foreground,
        hover: theme.ghostHover }
    case 'destructive':
      return { background: theme.destructiveSurface, foreground: theme.destructive,
        hover: theme.destructiveHover }
    case 'link':
      return { background: 'transparent', foreground: theme.primary,
        hover: 'transparent' }
  }
}

/** Gives each button size the shadcn Nova height, inset, spacing, and text size. */
export function buttonSize(size: ButtonSize | undefined): { height: number; padding: number; gap: number; fontSize: number; icon: boolean } {
  switch (size ?? 'default') {
    case 'xs': return { height: 24, padding: 8, gap: 4, fontSize: 12, icon: false }
    case 'sm': return { height: 28, padding: 10, gap: 4, fontSize: 13, icon: false }
    case 'lg': return { height: 36, padding: 10, gap: 6, fontSize: 14, icon: false }
    case 'icon-xs': return { height: 24, padding: 0, gap: 4, fontSize: 12, icon: true }
    case 'icon-sm': return { height: 28, padding: 0, gap: 4, fontSize: 13, icon: true }
    case 'icon-lg': return { height: 36, padding: 0, gap: 6, fontSize: 14, icon: true }
    case 'icon': return { height: 32, padding: 0, gap: 6, fontSize: 14, icon: true }
    default: return { height: 32, padding: 10, gap: 6, fontSize: 14, icon: false }
  }
}

/** Resolves the smaller Nova corner radius used by extra-small and small buttons. */
export function buttonRadius(size: ButtonSize | undefined, baseRadius: number): number {
  switch (size) {
    case 'xs':
    case 'icon-xs': return Math.min(baseRadius * 0.8, 10)
    case 'sm':
    case 'icon-sm': return Math.min(baseRadius * 0.8, 12)
    default: return baseRadius
  }
}
