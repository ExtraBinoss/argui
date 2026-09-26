/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import { ReactButton } from '../../../../packages/widgets/src/react/button'
import { ReactSonner, toast } from '../../../../packages/widgets/src/react/sonner'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates the Sonner stack, notification tones, actions, and promises. */
export function SonnerPage(props: { theme: Palette }): ReactElement {
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
      <ReactButton id="sonner-default" label="Default" theme={props.theme} kind="secondary"
        onClick={() => toast('Event has been created', { description: 'The change is ready to review.' })} />
      <ReactButton id="sonner-success" label="Success" theme={props.theme} kind="secondary"
        onClick={() => toast.success('Profile updated', { description: 'Your changes were saved.' })} />
      <ReactButton id="sonner-info" label="Info" theme={props.theme} kind="secondary"
        onClick={() => toast.info('Reminder', { description: 'The meeting starts in ten minutes.' })} />
      <ReactButton id="sonner-warning" label="Warning" theme={props.theme} kind="secondary"
        onClick={() => toast.warning('Check the schedule', { description: 'The selected time is outside office hours.' })} />
      <ReactButton id="sonner-error" label="Error" theme={props.theme} kind="secondary"
        onClick={() => toast.error('Upload failed', { description: 'Check the connection and try again.' })} />
      <ReactButton id="sonner-action" label="With action" theme={props.theme} kind="secondary"
        onClick={() => toast('Invitation sent', { action: { label: 'Undo', onClick: () => toast('Invitation recalled') } })} />
      <ReactButton id="sonner-promise" label="Promise" theme={props.theme} kind="primary" onClick={showPromise} />
      <ReactButton id="sonner-persistent" label="Persistent" theme={props.theme} kind="secondary"
        onClick={() => toast.loading('Waiting for confirmation', { description: 'Dismiss this notice when ready.' })} />
      <ReactButton id="sonner-dismiss-all" label="Dismiss all" theme={props.theme} kind="quiet"
        onClick={() => toast.dismiss()} />
    </row>
    <ReactSonner theme={props.theme} />
  </column>
}
