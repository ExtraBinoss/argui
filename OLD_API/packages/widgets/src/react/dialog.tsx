/** @jsxImportSource @argui/react */
import type { ReactElement, ReactNode } from 'react'
import type { DialogProps as SharedDialogProps } from '../shared/types'
import { ReactButton } from './button'

export type ReactDialogProps = SharedDialogProps<ReactNode>

/** Places the same centered, focus-trapped modal through the React adapter. */
export function ReactDialog(props: ReactDialogProps): ReactElement | null {
  if (!props.open) return null
  const close = () => props.onOpenChange(false)
  return <popupWindow nativeKey={`${props.id}-dialog`} placement="fill"
    width="fill" height="fill" window_layer="modal" containment="modal"
    dismiss_policy="escape" initial_focus={props.closeLabel === false ? 'first' : `${props.id}-close`} restore_focus={true}
    role="dialog" accessible_name={props.title} onDismiss={close}>
    <rectangle width="fill" height="fill" background={props.scrimColor ?? '#090c1c99'}
      backdrop_filter={`blur(${props.blur ?? 16}px)`}>
      <column width="fill" height="fill" align_items="center" justify_content="center" padding={16}>
        <rectangle width={props.width ?? 420} background={props.surfaceColor ?? props.theme.surface}
          border_color={props.theme.border} border_width={1} radius={props.radius ?? props.theme.dialogRadius}
          shadow_blur={props.theme.dialogShadowBlur} shadow_offset_y={8} shadow_color={props.theme.overlayShadow}>
          <column width="fill" gap={16} padding={props.padding ?? props.theme.dialogPadding}>
            <text width="fill" text={props.title} color={props.theme.foreground} font_size={20} weight={600} />
            {props.description ? <text width="fill" text={props.description} color={props.theme.muted} font_size={14} /> : null}
            {props.children}
            {props.closeLabel !== false ? <row width="fill" justify_content="end">
              <ReactButton id={`${props.id}-close`} label={props.closeLabel ?? 'Close'} theme={props.theme}
                kind="primary" onClick={close} />
            </row> : null}
          </column>
        </rectangle>
      </column>
    </rectangle>
  </popupWindow>
}
