/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import type { ProgressOptions } from '../shared/new-controls'

/** Props for a themed, read-only progress indicator. */
export type ProgressProps = ProgressOptions

/** Renders determinate or indeterminate progress with native accessibility data. */
export function Progress(props: ProgressProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const max = Number.isFinite(props.max) && (props.max ?? 100) > 0 ? props.max! : 100
  const value = props.value === null ? null : Math.max(0, Math.min(max, Number.isFinite(props.value) ? props.value : 0))
  const fraction = value === null ? 0 : value / max
  const sweepDistance = typeof props.width === 'number' && Number.isFinite(props.width)
    ? Math.max(0, props.width * 0.7) : 168
  return <container id={props.id} role="progress" accessibleName={props.accessibleName}
    accessibleValue={value === null ? 'In progress' : undefined}
    numericValue={value ?? undefined} minimumValue={value === null ? undefined : 0}
    maximumValue={value === null ? undefined : max} busy={value === null}
    width={props.width ?? '100%'} height={props.height ?? 8}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}>
    <rectangle width="100%" height="100%" radii={4} background={theme.muted} clip={true}>
      {value === null
        ? <rectangle width="30%" height="100%" background={theme.primary} radii={4}
          loopMs={900} loopTranslateX={sweepDistance} loopPlaying={props.playing !== false} />
        : <rectangle width={`${fraction * 100}%`} height="100%" background={theme.primary} radii={4} />}
    </rectangle>
  </container>
}
