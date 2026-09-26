/** @jsxImportSource @argui/react */
import { useSyncExternalStore, type ReactElement } from 'react'
import type { Palette } from '../shared/theme'
import {
  createToastStore, notificationStore, toast,
  type ToastRecord, type ToastStore,
} from '../shared/notification-stack'
import { ReactToast } from './toast'

export { createToastStore, toast }
export { ReactToast }
export type { ReactToastProps } from './toast'
export type {
  ToastAction, ToastApi, ToastOptions, ToastPromiseMessages, ToastRecord, ToastVariant,
} from '../shared/notification-stack'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Available corners for the floating notification stack. */
export type SonnerPosition = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'

/** Props for the shared notification stack and live-region host. */
export interface ReactSonnerProps {
  /** Resolved application palette. */
  theme: Palette
  /** Isolated store to display; defaults to the shared store used by `toast`. */
  store?: ToastStore
  /** Corner where the stack is positioned. Defaults to bottom-right. */
  position?: SonnerPosition
  /** Maximum number of most-recent notices shown at once. Defaults to five. */
  maxVisible?: number
  /** Accessible group name for the notification stack. */
  label?: string
  /** Width of each notice stack in logical pixels. Defaults to 360. */
  width?: number
}

/** Mounts the shared notification stack, timers, actions, and accessible announcements. */
export function ReactSonner(props: ReactSonnerProps): ReactElement {
  const store = props.store ?? notificationStore
  const records = useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot)
  const position = props.position ?? 'bottom-right'
  const top = position.startsWith('top')
  const left = position.endsWith('left')
  const maxVisible = Math.max(1, Math.floor(props.maxVisible ?? 5))
  const visible = records.slice(-maxVisible)

  return <container nativeKey="argui-sonner" position="absolute" z_index={1000}
    inset_top={top ? 18 : undefined} inset_bottom={top ? undefined : 18}
    inset_left={left ? 18 : undefined} inset_right={left ? undefined : 18}
    width={props.width ?? (mobile ? 320 : 360)} max_height={560} scroll_y={true}
    visible={visible.length > 0} role="group" accessible_name={props.label ?? 'Notifications'}>
    <column width="fill" gap={10}>
      {visible.map((notification) => <ReactToast key={notification.id} theme={props.theme}
        notification={notification} store={store} />)}
    </column>
  </container>
}
