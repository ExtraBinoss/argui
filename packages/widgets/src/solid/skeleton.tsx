import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'

/** Props for a themed, optionally pulsing placeholder block. */
export interface SkeletonProps {
  /** Palette used for the placeholder and its pulse colors. */
  theme: Palette
  /** Width in logical pixels, or `fill` to occupy the available width. */
  width?: number | 'fill'
  /** Height in logical pixels. */
  height?: number
  /** Corner radius in logical pixels. */
  radius?: number
  /** Whether to pulse between raised surface and surface colors. Defaults to true. */
  animated?: boolean
}

/** Renders a decorative placeholder block using native Argui primitives. */
export function Skeleton(props: SkeletonProps): JSX.Element {
  const animated = props.animated !== false
  return (
    <rectangle
      accessible_hidden={true}
      width={props.width ?? 'fill'}
      height={props.height ?? 16}
      radius={props.radius ?? props.theme.controlRadius}
      background={props.theme.surfaceRaised}
      loop_ms={animated ? 1100 : undefined}
      loop_background={animated ? props.theme.surface : undefined}
    />
  )
}
