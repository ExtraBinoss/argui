import type { JSX } from '@argui/solid/jsx-runtime'
import type { DialogProps as SharedDialogProps } from '../shared/types'
import { Button } from './button'

export type DialogProps = SharedDialogProps<JSX.Element>

/** Places a themed modal above a full-window blurred scrim and traps focus. */
export function Dialog(props: DialogProps): JSX.Element {
  const close = () => props.onOpenChange(false)
  return <>{props.open ? <popupWindow key={`${props.id}-dialog`} placement="fill"
    width="fill" height="fill" window_layer="modal" containment="modal"
    dismiss_policy="escape" initial_focus={`${props.id}-close`} restore_focus={true}
    role="dialog" accessible_name={props.title} onDismiss={close}>
    <rectangle width="fill" height="fill" background={props.scrimColor ?? '#090c1c99'}
      backdrop_filter={`blur(${props.blur ?? 16}px)`}>
      <column width="fill" height="fill" align_items="center" justify_content="center" padding={16}>
        <rectangle width={props.width ?? 420} background={props.surfaceColor ?? props.theme.surface}
          border_color={props.theme.border} border_width={1} radius={props.radius ?? 16}
          shadow_blur={24} shadow_offset_y={8} shadow_color={props.theme.overlayShadow}>
          <column width="fill" gap={16} padding={props.padding ?? 24}>
            <text width="fill" text={props.title} color={props.theme.foreground} font_size={20} weight={600} />
            {props.children}
            <row width="fill" justify_content="end">
              <Button id={`${props.id}-close`} label={props.closeLabel ?? 'Close'} theme={props.theme}
                kind="primary" onClick={close} />
            </row>
          </column>
        </rectangle>
      </column>
    </rectangle>
  </popupWindow> : null}</>
}
