import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'
import { notificationStore, type ToastRecord, type ToastStore } from '../shared/notification-stack'
import { Button } from './button'

export type {
  ToastAction, ToastApi, ToastOptions, ToastPromiseMessages, ToastRecord, ToastVariant,
} from '../shared/notification-stack'

/** Props for one notification view; lifetime and stack ordering belong to Sonner. */
export interface ToastProps {
  /** Resolved application palette. */
  theme: Palette
  /** Notification content supplied by a Sonner store or by a standalone preview. */
  notification: ToastRecord
  /** Store used for hover-pausing; defaults to the shared Sonner store. */
  store?: ToastStore
  /** Dismisses the current item; defaults to the supplied store. */
  onDismiss?: (id: string) => void
  /** Pauses the store timer while the pointer is over this toast. Defaults to true. */
  pauseOnHover?: boolean
}

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Renders one themed, accessible notification item for use by Sonner. */
export function Toast(props: ToastProps): JSX.Element {
  const notice = () => props.notification
  const store = () => props.store ?? notificationStore
  const urgent = () => notice().variant === 'error' || notice().variant === 'warning'
  const tone = () => urgent() ? props.theme.destructive
    : notice().variant === 'success' ? props.theme.accent : props.theme.foreground
  const symbol = () => notice().variant === 'success' ? '✓'
    : notice().variant === 'error' ? '×'
      : notice().variant === 'warning' ? '!' : notice().variant === 'loading' ? '…' : 'i'
  const dismiss = () => props.onDismiss ? props.onDismiss(notice().id) : store().dismiss(notice().id)
  const runAction = () => {
    try { notice().action?.onClick() } finally { dismiss() }
  }
  const pause = () => { if (props.pauseOnHover !== false && !mobile) store().pause(notice().id) }
  const resume = () => { if (props.pauseOnHover !== false && !mobile) store().resume(notice().id) }

  return <focusScope key={notice().id} role={urgent() ? 'alert' : 'status'}
    accessible_name={notice().title} accessible_description={notice().description}
    live={urgent() ? 'assertive' : 'polite'} busy={notice().variant === 'loading'}>
    <touchArea width="fill" onPointerEnter={pause} onPointerLeave={resume}>
      <rectangle width="fill" background={props.theme.overlaySurface} border_color={urgent() ? tone() : props.theme.border}
        border_width={1} radius={props.theme.overlayRadius}>
        <row width="fill" gap={10} padding={props.theme.overlayPadding} align_items="start">
          <container width={22} height={22} background={props.theme.surfaceRaised} radius={11}>
            <text width="fill" height="fill" text={symbol()} color={tone()} font_size={14} weight={700}
              role="text" accessible_hidden={true} />
          </container>
          <column width="fill" gap={4}>
            <text width="fill" text={notice().title} color={props.theme.foreground}
              font_size={props.theme.controlFontSize} weight={600} />
            {notice().description ? <text width="fill" text={notice().description} color={props.theme.muted}
              font_size={13} line_height={1.35} /> : null}
            {notice().action ? <Button id={`${notice().id}-action`} label={notice().action!.label}
              theme={props.theme} kind="quiet" size="sm" onClick={runAction} /> : null}
          </column>
          {notice().dismissible ? <focusScope key={`${notice().id}-dismiss`} role="button"
            accessible_name="Dismiss notification" focusable={true} keyboard_activation="enter_or_space"
            onClick={dismiss}>
            <rectangle width={26} height={26} background={props.theme.surfaceRaised} radius={13}>
              <text width="fill" height="fill" text="×" color={props.theme.muted} font_size={18}
                role="text" accessible_hidden={true} />
            </rectangle>
          </focusScope> : null}
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}
