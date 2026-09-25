/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Switch, type Palette } from '@argui/widgets/react'

/** Demonstrates controlled, uncontrolled, small and disabled switches. */
export function SwitchPage(props: { theme: Palette }): ReactElement {
  const [enabled, setEnabled] = useState(false)
  return <column width="fill" gap={16}>
    <row gap={12} align_items="center">
      <Switch id="switch-notifications" label="Notifications" theme={props.theme}
        checked={enabled} onCheckedChange={setEnabled} />
      <text text={`Notifications ${enabled ? 'on' : 'off'}`}
        color={props.theme.foreground} font_size={14} />
    </row>
    <row gap={12} align_items="center">
      <Switch id="switch-compact" label="Compact mode" theme={props.theme}
        size="sm" defaultChecked />
      <text text="Uncontrolled compact mode" color={props.theme.muted} font_size={13} />
    </row>
    <row gap={12} align_items="center">
      <Switch id="switch-disabled" label="Locked option" theme={props.theme}
        disabled checked />
      <text text="Unavailable" color={props.theme.muted} font_size={13} />
    </row>
  </column>
}
