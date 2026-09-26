/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import type { Palette } from '../shared/theme'

/** Props for a themed visible form label with an optional control relation. */
export interface ReactLabelProps {
  /** Visible label text and its accessible name. */
  text: string
  /** Palette used for label typography. */
  theme: Palette
  /** Target control key; the matching label key becomes `${htmlFor}-label`. */
  htmlFor?: string
  /** Explicit semantic key, overriding the key derived from `htmlFor`. */
  id?: string
  /** Whether the related control is disabled. */
  disabled?: boolean
  /** Whether to show a required marker after the text. */
  required?: boolean
}

/** Renders a native text label that can be referenced by a control's `labelled_by`. */
export function ReactLabel(props: ReactLabelProps): ReactElement {
  const key = props.id ?? (props.htmlFor ? `${props.htmlFor}-label` : undefined)
  return (
    <row nativeKey={key} role="text" accessible_name={props.text} align_items="center" gap={4}
      opacity={props.disabled ? 0.52 : 1}>
      <text text={props.text} color={props.theme.foreground} font_size={props.theme.controlFontSize}
        weight={500} accessible_hidden={true} />
      {props.required ? <text text="*" color={props.theme.destructive} font_size={props.theme.controlFontSize}
        accessible_hidden={true} /> : null}
    </row>
  )
}
