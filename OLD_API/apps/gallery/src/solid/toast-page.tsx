import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button } from '../../../../packages/widgets/src/solid/button'
import { Toast } from '../../../../packages/widgets/src/solid/toast'
import type { ToastRecord } from '../../../../packages/widgets/src/shared/notification-stack'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates Toast as the standalone renderer used by the Sonner stack. */
export function ToastPage(props: { theme: Palette }): JSX.Element {
  const [visible, setVisible] = createSignal(true)
  const [message, setMessage] = createSignal('Saved changes can be undone from the notice.')
  const notification: ToastRecord = {
    id: 'toast-renderer-preview',
    title: 'Your profile was updated',
    description: 'The new profile details are available to your team.',
    variant: 'success',
    dismissible: true,
    action: { label: 'Undo', onClick: () => setMessage('Undo was activated.') },
  }

  return <column width="fill" gap={14}>
    <text text="Toast renders one notice. Sonner owns the shared stack, action lifecycle, and expiry timer."
      color={props.theme.muted} font_size={13} />
    {visible() ? <Toast theme={props.theme} notification={notification}
      onDismiss={() => setVisible(false)} /> : null}
    <text text={message()} color={props.theme.foreground} font_size={14} />
    <Button id="toast-show-preview" label="Show toast preview" theme={props.theme} kind="secondary"
      onClick={() => { setVisible(true); setMessage('Saved changes can be undone from the notice.') }} />
  </column>
}
