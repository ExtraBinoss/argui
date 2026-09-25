import type { JSX } from '@argui/solid/jsx-runtime'
import { Kbd, KbdGroup, type Palette } from '@argui/widgets/solid'

/** Shows standalone keycaps and grouped shortcut notation. */
export function KbdPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={16}>
    <text text="Keyboard shortcuts use themed keycaps that group naturally in text and controls."
      width="fill" color={props.theme.muted} font_size={14} />
    <row width="fill" gap={16} align_items="center">
      <KbdGroup label="Modifier keys">
        <Kbd theme={props.theme} label="⌘" />
        <Kbd theme={props.theme} label="⇧" />
        <Kbd theme={props.theme} label="⌥" />
        <Kbd theme={props.theme} label="⌃" />
      </KbdGroup>
      <KbdGroup label="Bold shortcut">
        <Kbd theme={props.theme} label="Ctrl" />
        <text text="+" color={props.theme.muted} font_size={12} />
        <Kbd theme={props.theme} label="B" />
      </KbdGroup>
      <KbdGroup label="Accept shortcut">
        <Kbd theme={props.theme} label="Enter" />
      </KbdGroup>
    </row>
  </column>
}
