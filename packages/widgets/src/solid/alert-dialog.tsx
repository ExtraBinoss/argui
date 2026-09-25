import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { AlertDialogProps as SharedAlertDialogProps } from '../shared/surface-d'
import { Button } from './button'

export type AlertDialogProps = SharedAlertDialogProps

/** Asks for explicit confirmation in an accessible modal alert dialog. */
export function AlertDialog(props: AlertDialogProps): JSX.Element {
  const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false)
  const opened = () => props.open ?? localOpen()
  const requestOpen = (value: boolean) => {
    if (opened() === value) return
    if (props.open === undefined) setLocalOpen(value)
    props.onOpenChange?.(value)
  }
  const close = () => requestOpen(false)
  const cancel = () => {
    props.onCancel?.()
    close()
  }
  const confirm = () => {
    if (props.confirmDisabled) return
    props.onConfirm()
    close()
  }
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 440
  const popupKey = `${props.id}-alert-dialog`
  return <>
    <Button id={`${props.id}-trigger`} label={props.triggerLabel} theme={props.theme}
      kind="outline" expanded={opened()} controls={popupKey} onClick={() => requestOpen(true)} />
    {opened() ? <popupWindow key={popupKey} placement="fill" width="fill" height="fill"
      window_layer="modal" containment="modal" dismiss_policy="escape"
      initial_focus={`${props.id}-cancel`} restore_focus={true} role="alert_dialog" modal={true}
      accessible_name={props.title} accessible_description={props.description}
      onDismiss={cancel}>
      <rectangle width="fill" height="fill" background="#090c1c99" backdrop_filter="blur(12px)">
        <column width="fill" height="fill" align_items="center" justify_content="center" padding={16}>
          <rectangle width={width} background={props.theme.surface} border_color={props.theme.border}
            border_width={1} radius={props.theme.dialogRadius} shadow_blur={props.theme.dialogShadowBlur}
            shadow_offset_y={8} shadow_color={props.theme.overlayShadow}>
            <column width="fill" gap={16} padding={props.theme.dialogPadding}>
              <text width="fill" text={props.title} color={props.theme.foreground} font_size={20} weight={600} />
              <text width="fill" text={props.description} color={props.theme.muted} font_size={14} />
              <row width="fill" gap={10} justify_content="end">
                <Button id={`${props.id}-cancel`} label={props.cancelLabel ?? 'Cancel'} theme={props.theme}
                  kind="outline" onClick={cancel} />
                <Button id={`${props.id}-confirm`} label={props.confirmLabel} theme={props.theme}
                  kind={props.destructive === false ? 'primary' : 'destructive'}
                  disabled={props.confirmDisabled} onClick={confirm} />
              </row>
            </column>
          </rectangle>
        </column>
      </rectangle>
    </popupWindow> : null}
  </>
}
