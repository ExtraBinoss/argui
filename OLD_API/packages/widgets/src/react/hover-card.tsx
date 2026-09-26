/** @jsxImportSource @argui/react */
import { useEffect, useMemo, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { createSurfaceActivity, type SurfaceHoverCardProps } from '../shared/surface-c'

export type ReactHoverCardProps = SurfaceHoverCardProps<ReactNode, ReactNode>

/** Shows the same rich, non-modal card while its trigger is hovered or focused. */
export function ReactHoverCard(props: ReactHoverCardProps): ReactElement {
  const latestProps = useRef(props)
  latestProps.current = props
  const [localOpen, setLocalOpen] = useState(props.defaultOpen ?? false)
  const currentOpen = useRef(props.open ?? localOpen)
  currentOpen.current = props.open ?? localOpen
  const activity = useMemo(() => createSurfaceActivity((value) => {
    if (currentOpen.current === value) return
    currentOpen.current = value
    const current = latestProps.current
    if (current.open === undefined) setLocalOpen(value)
    current.onOpenChange?.(value)
  }, 300, 180), [])
  useEffect(() => () => activity.dispose(), [activity])
  const opened = props.open ?? localOpen
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 320
  const popupKey = `${props.id}-hover-card`
  return <>
    <focusScope nativeKey={props.id} role="generic" controls={popupKey} expanded={opened}
      focus_on_tab_navigation={false} onFocus={activity.focusEnter} onBlur={activity.focusLeave}>
      <touchArea nativeKey={`${props.id}-hover-card-trigger`} enabled={true}
        onPointerEnter={activity.triggerEnter} onPointerLeave={activity.triggerLeave}>
        {props.children}
      </touchArea>
    </focusScope>
    {opened ? <popupWindow nativeKey={popupKey} anchor={props.id} placement={props.placement ?? 'bottom'}
      width={width} window_layer="popover" dismiss_policy="outside_pointer_or_escape"
      containment="none" restore_focus={false} role="group" accessible_name={props.label}
      onDismiss={activity.dismiss}>
      <focusScope nativeKey={`${popupKey}-content-focus`} role="generic"
        focus_on_tab_navigation={false} onFocus={activity.focusEnter} onBlur={activity.focusLeave}>
        <touchArea nativeKey={`${popupKey}-content-hover`} enabled={true} mouse_cursor="default"
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
