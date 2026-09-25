import type { JSX } from '@argui/solid/jsx-runtime'
import type { ProgressProps } from '../shared/foundation-e'

/** Shows a themed determinate value or an animated indeterminate progress state. */
export function Progress(props: ProgressProps): JSX.Element {
  const height = Number.isFinite(props.height) && (props.height ?? 0) > 0 ? props.height! : 8
  const requestedMax = props.max ?? 100
  const max = Number.isFinite(requestedMax) && requestedMax > 0 ? requestedMax : 100
  const indeterminate = props.value == null || !Number.isFinite(props.value)
  const value = indeterminate ? undefined : Math.max(0, Math.min(max, props.value!))
  const fraction = value === undefined ? 0 : value / max
  const movingWidth = Math.min(typeof props.width === 'number' ? props.width : 72, Math.max(24, height * 3))
  const travel = typeof props.width === 'number' ? Math.max(0, props.width - movingWidth) : 180

  return <container width={props.width ?? 'fill'} height={height} clip radius={height / 2}
    background={props.theme.surfaceRaised} role="progress"
    accessible_name={props.label ?? 'Progress'} accessible_value={indeterminate ? 'In progress' : undefined}
    numeric_value={value} minimum_value={0} maximum_value={max}>
    {indeterminate
      ? <rectangle width={movingWidth} height="fill" background={props.theme.accent}
        loop_ms={1200} loop_translate_x={travel} />
      : <row width="fill" height="fill">
        {fraction > 0 ? <container grow={fraction} height="fill"
          radius={height / 2} background={props.theme.accent} transition_ms={180} /> : null}
        {fraction < 1 ? <container grow={1 - fraction} height="fill" /> : null}
      </row>}
  </container>
}
