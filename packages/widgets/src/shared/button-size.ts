import type { Palette } from './theme'

/** Native button sizes corresponding to the shadcn control density scale. */
export type ButtonSize = 'xs' | 'sm' | 'default' | 'lg' | 'icon' | 'icon-xs' | 'icon-sm' | 'icon-lg'

/** Computes themed dimensions for a button size in logical pixels. */
export function buttonSize(size: ButtonSize, theme: Palette): {
  height: number; padding: number; gap: number; icon: number; font: number; iconOnly: boolean
} {
  const iconOnly = size.startsWith('icon')
  const height = size === 'xs' || size === 'icon-xs' ? 24
    : size === 'sm' || size === 'icon-sm' ? 32
      : size === 'lg' || size === 'icon-lg' ? 40 : 36
  const padding = iconOnly ? 0 : size === 'xs' ? 8 : size === 'sm' ? 12
    : size === 'lg' ? 24 : Math.max(12, theme.controlPadding)
  return {
    height,
    padding,
    gap: size === 'xs' || size === 'sm' || size === 'icon-xs' || size === 'icon-sm' ? 5 : 8,
    icon: size === 'xs' || size === 'icon-xs' ? 12 : size === 'sm' || size === 'icon-sm' ? 14 : 18,
    font: size === 'xs' ? 12 : size === 'sm' ? 13 : theme.controlFontSize,
    iconOnly,
  }
}
