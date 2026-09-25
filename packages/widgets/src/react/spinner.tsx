/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import type { Palette } from '../shared/theme'

const dots = [
  [0, -1, 1], [Math.SQRT1_2, -Math.SQRT1_2, 0.9], [1, 0, 0.8],
  [Math.SQRT1_2, Math.SQRT1_2, 0.7], [0, 1, 0.6],
  [-Math.SQRT1_2, Math.SQRT1_2, 0.5], [-1, 0, 0.4],
  [-Math.SQRT1_2, -Math.SQRT1_2, 0.3],
] as const

/** Props for the accessible native loading indicator. */
export interface ReactSpinnerProps {
  /** Palette used for the default spinner color. */
  theme: Palette
  /** Square spinner size in logical pixels. Defaults to 16. */
  size?: number
  /** Accessible status name announced by assistive technology. Defaults to `Loading`. */
  label?: string
  /** Dot color override. Defaults to the palette's muted foreground. */
  color?: string
  /** Whether the indicator rotates. Defaults to true. */
  animated?: boolean
}

/** Renders a rotating, eight-dot loading status with a native animation. */
export function ReactSpinner(props: ReactSpinnerProps): ReactElement {
  const size = Math.max(8, props.size ?? 16)
  const dotSize = Math.max(2, size * 0.14)
  const orbit = size / 2 - dotSize / 2
  const rotation = props.animated === false ? undefined : 900
  return (
    <rectangle
      role="status"
      accessible_name={props.label ?? 'Loading'}
      busy={true}
      live="polite"
      width={size}
      height={size}
      rotation_loop_ms={rotation}
    >
      {dots.map(([x, y, opacity], index) => (
        <rectangle
          key={`spinner-dot-${index}`}
          nativeKey={`spinner-dot-${index}`}
          width={dotSize}
          height={dotSize}
          x={size / 2 + x * orbit - dotSize / 2}
          y={size / 2 + y * orbit - dotSize / 2}
          radius={dotSize / 2}
          background={props.color ?? props.theme.muted}
          opacity={opacity}
          accessible_hidden={true}
        />
      ))}
    </rectangle>
  )
}
