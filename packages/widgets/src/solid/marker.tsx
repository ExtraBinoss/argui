import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'

/** Visual treatment for a short section marker. */
export type MarkerVariant = 'default' | 'separator' | 'border'

/** Props for a themed marker row. */
export interface MarkerProps {
  theme: Palette
  children: JSX.Element
  variant?: MarkerVariant
  label?: string
}

/** Arranges a marker with optional separator lines or a bottom border. */
export function Marker(props: MarkerProps): JSX.Element {
  const variant = () => props.variant ?? 'default'
  return <column width="fill" gap={7} role="group" accessible_name={props.label}>
    <row width="fill" min_height={20} gap={8} align_items="center">
      {variant() === 'separator' ? <container grow={1}><rectangle width="fill" height={1} background={props.theme.border} accessible_hidden={true} /></container> : null}
      {props.children}
      {variant() === 'separator' ? <container grow={1}><rectangle width="fill" height={1} background={props.theme.border} accessible_hidden={true} /></container> : null}
    </row>
    {variant() === 'border' ? <rectangle width="fill" height={1} background={props.theme.border} accessible_hidden={true} /> : null}
  </column>
}

/** Props for a decorative icon or custom media within a marker. */
export interface MarkerIconProps {
  children: JSX.Element
}

/** Reserves a compact leading slot for a marker icon. */
export function MarkerIcon(props: MarkerIconProps): JSX.Element {
  return <container width={16} height={16} accessible_hidden={true}>{props.children}</container>
}

/** Props for the label or caller-composed content of a marker. */
export interface MarkerContentProps {
  children: JSX.Element
}

/** Groups caller-composed marker content without imposing a text color. */
export function MarkerContent(props: MarkerContentProps): JSX.Element {
  return <row min_width={0} gap={4} align_items="center">{props.children}</row>
}
