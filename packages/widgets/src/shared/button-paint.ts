import type { WidgetTheme } from './theme'
import type { ButtonSize, ButtonVariant } from './types'

/** Resolves native button colors from shadcn Neutral roles without JavaScript hover state. */
export function buttonPaint(variant: ButtonVariant | undefined, theme: Readonly<WidgetTheme>, selected = false) {
  switch (variant ?? 'default') {
    case 'default':
      return { background: theme.primary, foreground: theme.primaryForeground,
        hover: theme.primaryHover, pressed: theme.primaryHover }
    case 'outline':
      return { background: theme.outlineSurface, foreground: theme.foreground,
        hover: theme.outlineHover, pressed: theme.outlineHover }
    case 'secondary':
      return { background: theme.secondary, foreground: theme.secondaryForeground,
        hover: theme.secondaryHover, pressed: theme.secondaryHover }
    case 'ghost':
      return { background: selected ? theme.accent : 'transparent',
        foreground: selected ? theme.primary : theme.foreground,
        hover: theme.ghostHover, pressed: theme.ghostHover }
    case 'destructive':
      return { background: theme.destructiveSurface, foreground: theme.destructive,
        hover: theme.destructiveHover, pressed: theme.destructiveHover }
    case 'link':
      return { background: 'transparent', foreground: theme.primary,
        hover: 'transparent', pressed: 'transparent' }
  }
}

/** Gives each button size the shadcn Nova height and matching horizontal inset. */
export function buttonSize(size: ButtonSize | undefined): { height: number; padding: number; icon: boolean } {
  switch (size ?? 'default') {
    case 'xs': return { height: 24, padding: 8, icon: false }
    case 'sm': return { height: 28, padding: 10, icon: false }
    case 'lg': return { height: 36, padding: 16, icon: false }
    case 'icon-xs': return { height: 24, padding: 0, icon: true }
    case 'icon-sm': return { height: 28, padding: 0, icon: true }
    case 'icon-lg': return { height: 36, padding: 0, icon: true }
    case 'icon': return { height: 32, padding: 0, icon: true }
    default: return { height: 32, padding: 12, icon: false }
  }
}
