import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { onCleanup } from 'solid-js'
import type { Palette } from '../shared/theme'
import {
  createToastStore, notificationStore, toast,
  type ToastRecord, type ToastStore,
} from '../shared/notification-stack'
import { Toast } from './toast'

export { createToastStore, toast }
export { Toast }
export type {
  ToastProps,
} from './toast'
export type {
  ToastAction, ToastApi, ToastOptions, ToastPromiseMessages, ToastRecord, ToastVariant,
} from '../shared/notification-stack'

/** Available corners for the floating notification stack. */
export type SonnerPosition = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'

/** Props for the shared notification stack and live-region host. */
export interface SonnerProps {
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
export function Sonner(props: SonnerProps): JSX.Element {
  const store = () => props.store ?? notificationStore
  const subscribedStore = store()
  const [records, setRecords] = createSignal<readonly ToastRecord[]>(subscribedStore.getSnapshot())
  const unsubscribe = subscribedStore.subscribe(() => setRecords(subscribedStore.getSnapshot()))
  onCleanup(unsubscribe)
  const position = () => props.position ?? 'bottom-right'
  const top = () => position().startsWith('top')
  const left = () => position().endsWith('left')
  const maxVisible = () => Math.max(1, Math.floor(props.maxVisible ?? 5))
  const visible = () => records().slice(-maxVisible())

  return <container key="argui-sonner" position="absolute" z_index={1000}
    inset_top={top() ? 18 : undefined} inset_bottom={top() ? undefined : 18}
    inset_left={left() ? 18 : undefined} inset_right={left() ? undefined : 18}
    width={props.width ?? ((globalThis as { __arguiMobile?: boolean }).__arguiMobile === true ? 320 : 360)}
    max_height={560} scroll_y={true}
    visible={visible().length > 0} role="group" accessible_name={props.label ?? 'Notifications'}>
    <column width="fill" gap={10}>
      {visible().map((notification) => <Toast theme={props.theme} notification={notification} store={store()} />)}
    </column>
  </container>
}
