import type { AssetRef } from '@argui/host'
import type { Palette } from './theme'

/** Shared sizes accepted by avatar primitives. */
export type AvatarSize = 'sm' | 'default' | 'lg' | number

/** Properties common to the Solid and React avatar adapters. */
interface AvatarBaseProps {
  theme: Palette
  size?: AvatarSize
  alt?: string
  label?: string
  bordered?: boolean
  badge?: { label: string; color?: string }
}

/** Requires fallback text when there is no registered image asset. */
export type AvatarProps = AvatarBaseProps & (
  | { source: AssetRef; fallback?: string }
  | { source?: undefined; fallback: string }
)

/** Image properties shared by the standalone avatar image helpers. */
export interface AvatarImageProps {
  theme: Palette
  source: AssetRef
  alt: string
  size?: AvatarSize
}

/** Fallback properties shared by the standalone avatar fallback helpers. */
export interface AvatarFallbackProps {
  theme: Palette
  text: string
  size?: AvatarSize
}

/** Badge properties shared by the standalone avatar badge helpers. */
export interface AvatarBadgeProps {
  theme: Palette
  color?: string
  size?: number
  avatarSize?: AvatarSize
  label?: string
}

/** Properties shared by avatar groups. */
export interface AvatarGroupProps<TChildren> {
  children: TChildren
  gap?: number
  overlap?: boolean
  label?: string
}

/** Properties shared by the avatar group count helpers. */
export interface AvatarGroupCountProps {
  theme: Palette
  count: number
  size?: AvatarSize
  label?: string
}

/** Shared aspect-ratio properties for the Solid and React adapters. */
export interface AspectRatioProps<TChildren> {
  children: TChildren
  ratio?: number
  width?: number | 'fill'
  theme: Palette
  background?: string
  radius?: number
  clip?: boolean
}

/** Shared separator properties for the Solid and React adapters. */
export interface SeparatorProps {
  theme: Palette
  orientation?: 'horizontal' | 'vertical'
  decorative?: boolean
  thickness?: number
  color?: string
}

/** Resolves the avatar's named size to its native pixel dimension. */
export function avatarDimension(size: AvatarSize = 'default'): number {
  if (typeof size === 'number') return Number.isFinite(size) && size > 0 ? size : 32
  return size === 'sm' ? 24 : size === 'lg' ? 40 : 32
}

/** Returns a valid positive ratio, falling back to a square for invalid input. */
export function validAspectRatio(ratio: number | undefined): number {
  return ratio !== undefined && Number.isFinite(ratio) && ratio > 0 ? ratio : 1
}
