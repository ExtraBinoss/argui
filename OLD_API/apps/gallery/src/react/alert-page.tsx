/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Button } from '@argui/widgets/react'
import type { Palette } from '@argui/widgets/react'
import { ReactAlert as Alert } from '../../../../packages/widgets/src/react/alert'

/** Demonstrates default, destructive, and conditionally rendered alerts. */
export function AlertPage(props: { theme: Palette }): ReactElement {
  const [showUpdate, setShowUpdate] = useState(false)
  return <column width="fill" gap={14}>
    <text text="Important feedback stays visible in the page and announces its urgency." color={props.theme.muted} font_size={14} />
    <Alert theme={props.theme} title="Changes saved" description="Your workspace settings have been updated." />
    <Alert theme={props.theme} variant="destructive" title="Unable to save changes"
      description="Check your connection and try again." />
    <Button id="alert-update-toggle" label={showUpdate ? 'Hide update notice' : 'Show update notice'}
      theme={props.theme} kind="outline" onClick={() => setShowUpdate((value) => !value)} />
    {showUpdate ? <Alert theme={props.theme} title="New version available"
      description="A restart will apply the latest improvements." /> : null}
  </column>
}
