import type { JSX } from '@argui/solid/jsx-runtime'
import { Button } from '../../../../packages/widgets/src/solid/button'
import { Sonner, toast } from '../../../../packages/widgets/src/solid/sonner'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates the Sonner stack, notification tones, actions, and promises. */
export function SonnerPage(props: { theme: Palette }): JSX.Element {
  const showPromise = () => {
    void toast.promise(
      new Promise<string>((resolve) => setTimeout(() => resolve('Draft'), 1400)),
      { loading: 'Saving draft…', success: (name) => `${name} saved`, error: 'Could not save draft' },
    )
  }

  return <column width="fill" gap={14}>
    <text text="Notifications share one stack. Hover pauses expiry; actions stay keyboard accessible."
      color={props.theme.muted} font_size={13} />
    <row width="fill" gap={8} wrap={true}>
      <Button id="sonner-default" label="Default" theme={props.theme} kind="secondary"
        onClick={() => toast('Event has been created', { description: 'The change is ready to review.' })} />
      <Button id="sonner-success" label="Success" theme={props.theme} kind="secondary"
        onClick={() => toast.success('Profile updated', { description: 'Your changes were saved.' })} />
      <Button id="sonner-info" label="Info" theme={props.theme} kind="secondary"
        onClick={() => toast.info('Reminder', { description: 'The meeting starts in ten minutes.' })} />
      <Button id="sonner-warning" label="Warning" theme={props.theme} kind="secondary"
        onClick={() => toast.warning('Check the schedule', { description: 'The selected time is outside office hours.' })} />
      <Button id="sonner-error" label="Error" theme={props.theme} kind="secondary"
        onClick={() => toast.error('Upload failed', { description: 'Check the connection and try again.' })} />
      <Button id="sonner-action" label="With action" theme={props.theme} kind="secondary"
        onClick={() => toast('Invitation sent', { action: { label: 'Undo', onClick: () => toast('Invitation recalled') } })} />
      <Button id="sonner-promise" label="Promise" theme={props.theme} kind="primary" onClick={showPromise} />
      <Button id="sonner-persistent" label="Persistent" theme={props.theme} kind="secondary"
        onClick={() => toast.loading('Waiting for confirmation', { description: 'Dismiss this notice when ready.' })} />
      <Button id="sonner-dismiss-all" label="Dismiss all" theme={props.theme} kind="quiet"
        onClick={() => toast.dismiss()} />
    </row>
    <Sonner theme={props.theme} />
  </column>
}
