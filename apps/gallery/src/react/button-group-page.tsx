/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText, type Palette } from '@argui/widgets/react'

/** Demonstrates horizontal, vertical, and labelled groups of native buttons. */
export function ButtonGroupPage(props: { theme: Palette }): ReactElement {
  const [lastAction, setLastAction] = useState('None')
  return <column width="fill" gap={16}>
    <text text="Button groups keep related controls together while every button remains independently focusable." color={props.theme.muted} font_size={14} />
    <ButtonGroup label="Message actions">
      <Button id="group-archive" label="Archive" theme={props.theme} kind="outline" onClick={() => setLastAction('Archive')} />
      <Button id="group-report" label="Report" theme={props.theme} kind="outline" onClick={() => setLastAction('Report')} />
      <Button id="group-snooze" label="Snooze" theme={props.theme} kind="outline" onClick={() => setLastAction('Snooze')} />
    </ButtonGroup>
    <ButtonGroup label="Display controls" gap={0}>
      <ButtonGroupText theme={props.theme} text="Zoom" />
      <ButtonGroupSeparator theme={props.theme} />
      <Button id="group-zoom-out" label="−" theme={props.theme} kind="secondary" onClick={() => setLastAction('Zoom out')} />
      <Button id="group-zoom-in" label="+" theme={props.theme} kind="secondary" onClick={() => setLastAction('Zoom in')} />
    </ButtonGroup>
    <ButtonGroup label="Vertical media controls" orientation="vertical">
      <Button id="group-up" label="Move up" theme={props.theme} kind="ghost" onClick={() => setLastAction('Move up')} />
      <Button id="group-down" label="Move down" theme={props.theme} kind="ghost" onClick={() => setLastAction('Move down')} />
    </ButtonGroup>
    <text text={`Last action: ${lastAction}`} color={props.theme.muted} font_size={12} />
  </column>
}
