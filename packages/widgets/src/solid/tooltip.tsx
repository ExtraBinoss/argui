import { createSignal, onCleanup } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createSurfaceActivity, type SurfaceTooltipProps } from '../shared/surface-c'

export type TooltipProps = SurfaceTooltipProps<JSX.Element, JSX.Element>

/** Shows a themed tooltip while its trigger is hovered or focused. */
export function Tooltip(props: TooltipProps): JSX.Element {
  const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false)
  const opened = () => props.open ?? localOpen()
  const requestOpen = (value: boolean) => {
    if (opened() === value) return
    if (props.open === undefined) setLocalOpen(value)
    props.onOpenChange?.(value)
  }
  const activity = createSurfaceActivity(requestOpen, 0, 140)
  onCleanup(() => activity.dispose())
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 220
  const popupKey = `${props.id}-tooltip`
  return <>
    <focusScope key={props.id} role="generic" described_by={opened() ? popupKey : undefined}
      focus_on_tab_navigation={false} onFocus={activity.focusEnter} onBlur={activity.focusLeave}>
      <touchArea key={`${props.id}-tooltip-trigger`} enabled={true}
        onPointerEnter={activity.triggerEnter} onPointerLeave={activity.triggerLeave}>
        {props.children}
      </touchArea>
    </focusScope>
    {opened() ? <popupWindow key={popupKey} anchor={props.id} placement={props.placement ?? 'top'}
      width={width} window_layer="popover" dismiss_policy="outside_pointer_or_escape"
      containment="none" restore_focus={false} role="tooltip"
      accessible_name={props.description} onDismiss={activity.dismiss}>
      <touchArea key={`${popupKey}-hover`} enabled={true} mouse_cursor="default"
        onPointerEnter={activity.contentEnter} onPointerLeave={activity.contentLeave}>
        <rectangle width={width} background={props.theme.foreground} border_color={props.theme.foreground}
          border_width={1} radius={props.theme.controlRadius}
          shadow_blur={props.theme.overlayShadowBlur} shadow_offset_y={3} shadow_color={props.theme.overlayShadow}>
          <column width="fill" padding={props.theme.controlPadding}>{props.content}</column>
        </rectangle>
      </touchArea>
    </popupWindow> : null}
  </>
}
