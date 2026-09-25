/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { ReactButton } from '../../../../packages/widgets/src/react/button'
import { ReactToast } from '../../../../packages/widgets/src/react/toast'
import type { ToastRecord } from '../../../../packages/widgets/src/shared/notification-stack'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates Toast as the standalone renderer used by the Sonner stack. */
export function ToastPage(props: { theme: Palette }): ReactElement {
  const [visible, setVisible] = useState(true)
  const [message, setMessage] = useState('Saved changes can be undone from the notice.')
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
    {visible ? <ReactToast theme={props.theme} notification={notification}
      onDismiss={() => setVisible(false)} /> : null}
    <text text={message} color={props.theme.foreground} font_size={14} />
    <ReactButton id="toast-show-preview" label="Show toast preview" theme={props.theme} kind="secondary"
      onClick={() => { setVisible(true); setMessage('Saved changes can be undone from the notice.') }} />
  </column>
}
