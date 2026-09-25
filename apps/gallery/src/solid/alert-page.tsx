import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, type Palette } from '@argui/widgets/solid'
import { Alert } from '../../../../packages/widgets/src/solid/alert'

/** Demonstrates default, destructive, and conditionally rendered alerts. */
export function AlertPage(props: { theme: Palette }): JSX.Element {
  const [showUpdate, setShowUpdate] = createSignal(false)
  return <column width="fill" gap={14}>
    <text text="Important feedback stays visible in the page and announces its urgency." color={props.theme.muted} font_size={14} />
    <Alert theme={props.theme} title="Changes saved" description="Your workspace settings have been updated." />
    <Alert theme={props.theme} variant="destructive" title="Unable to save changes"
      description="Check your connection and try again." />
    <Button id="alert-update-toggle" label={showUpdate() ? 'Hide update notice' : 'Show update notice'}
      theme={props.theme} kind="outline" onClick={() => setShowUpdate(!showUpdate())} />
    {showUpdate() ? <Alert theme={props.theme} title="New version available"
      description="A restart will apply the latest improvements." /> : null}
  </column>
}
