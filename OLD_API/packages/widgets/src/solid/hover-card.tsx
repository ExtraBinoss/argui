import { createSignal, onCleanup } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createSurfaceActivity, type SurfaceHoverCardProps } from '../shared/surface-c'

export type HoverCardProps = SurfaceHoverCardProps<JSX.Element, JSX.Element>

/** Shows a rich, non-modal card while its trigger is hovered or focused. */
export function HoverCard(props: HoverCardProps): JSX.Element {
  const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false)
  const opened = () => props.open ?? localOpen()
  const requestOpen = (value: boolean) => {
    if (opened() === value) return
    if (props.open === undefined) setLocalOpen(value)
    props.onOpenChange?.(value)
  }
  const activity = createSurfaceActivity(requestOpen, 300, 180)
  onCleanup(() => activity.dispose())
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 320
  const popupKey = `${props.id}-hover-card`
  return <>
    <focusScope key={props.id} role="generic" controls={popupKey} expanded={opened()}
      focus_on_tab_navigation={false} onFocus={activity.focusEnter} onBlur={activity.focusLeave}>
      <touchArea key={`${props.id}-hover-card-trigger`} enabled={true}
        onPointerEnter={activity.triggerEnter} onPointerLeave={activity.triggerLeave}>
        {props.children}
      </touchArea>
    </focusScope>
    {opened() ? <popupWindow key={popupKey} anchor={props.id} placement={props.placement ?? 'bottom'}
      width={width} window_layer="popover" dismiss_policy="outside_pointer_or_escape"
      containment="none" restore_focus={false} role="group" accessible_name={props.label}
      onDismiss={activity.dismiss}>
      <focusScope key={`${popupKey}-content-focus`} role="generic"
        focus_on_tab_navigation={false} onFocus={activity.focusEnter} onBlur={activity.focusLeave}>
        <touchArea key={`${popupKey}-content-hover`} enabled={true} mouse_cursor="default"
          onPointerEnter={activity.contentEnter} onPointerLeave={activity.contentLeave}>
          <rectangle width={width} background={props.theme.overlaySurface} border_color={props.theme.border}
            border_width={1} radius={props.theme.overlayRadius}
            backdrop_filter="blur(10px)" shadow_blur={props.theme.overlayShadowBlur}
            shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
            <column width="fill" gap={8} padding={props.theme.overlayPadding}>{props.content}</column>
          </rectangle>
        </touchArea>
      </focusScope>
    </popupWindow> : null}
  </>
}
