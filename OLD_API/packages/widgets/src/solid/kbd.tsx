import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'

/** Props for a themed keyboard keycap. */
export interface KbdProps {
  /** Palette used for the keycap background, border, and label. */
  theme: Palette
  /** Visible and accessible key label, such as `Ctrl`, `⌘`, or `Enter`. */
  label: string
}

/** Props for a row grouping related keyboard keycaps. */
export interface KbdGroupProps {
  /** Keycaps and optional Argui text separators displayed in the group. */
  children: JSX.Element
  /** Accessible name describing the shortcut group. */
  label?: string
  /** Space between group children in logical pixels. Defaults to 4. */
  gap?: number
}

/** Renders one keyboard key label in a raised, bordered keycap. */
export function Kbd(props: KbdProps): JSX.Element {
  return (
    <rectangle
      role="text"
      accessible_name={props.label}
      radius={4}
      border_color={props.theme.border}
      border_width={1}
      background={props.theme.surfaceRaised}
    >
      <row padding_left={5} padding_right={5} padding_top={2} padding_bottom={2}>
        <text text={props.label} color={props.theme.muted} font_size={12} weight={600} accessible_hidden={true} />
      </row>
    </rectangle>
  )
}

/** Renders a horizontally spaced group of keyboard keys and separators. */
export function KbdGroup(props: KbdGroupProps): JSX.Element {
  return (
    <row role="group" accessible_name={props.label} gap={props.gap ?? 4} align_items="center">
      {props.children}
    </row>
  )
}
