import type { JSX } from '@argui/solid/jsx-runtime'
import type { SeparatorProps } from '../shared/foundation-b'

/** Draws a themed horizontal or vertical divider with separator semantics. */
export function Separator(props: SeparatorProps): JSX.Element {
  const orientation = props.orientation ?? 'horizontal'
  const requestedThickness = props.thickness ?? 1
  const thickness = Number.isFinite(requestedThickness) && requestedThickness > 0
    ? requestedThickness : 1
  const decorative = props.decorative ?? true
  return <rectangle width={orientation === 'horizontal' ? 'fill' : thickness}
    height={orientation === 'horizontal' ? thickness : 'fill'}
    background={props.color ?? props.theme.border}
    role={decorative ? undefined : 'separator'} orientation={orientation}
    accessible_hidden={decorative} />
}
