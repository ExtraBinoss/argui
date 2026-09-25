/** @jsxImportSource @argui/react */
import { useEffect, useMemo, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { createSurfaceActivity, type SurfaceTooltipProps } from '../shared/surface-c'

export type ReactTooltipProps = SurfaceTooltipProps<ReactNode, ReactNode>

/** Shows the same themed tooltip while its trigger is hovered or focused. */
export function ReactTooltip(props: ReactTooltipProps): ReactElement {
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
  }, 0, 140), [])
  useEffect(() => () => activity.dispose(), [activity])
  const opened = props.open ?? localOpen
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 220
  const popupKey = `${props.id}-tooltip`
  return <>
    <focusScope nativeKey={props.id} role="generic" described_by={opened ? popupKey : undefined}
      focus_on_tab_navigation={false} onFocus={activity.focusEnter} onBlur={activity.focusLeave}>
      <touchArea nativeKey={`${props.id}-tooltip-trigger`} enabled={true}
        onPointerEnter={activity.triggerEnter} onPointerLeave={activity.triggerLeave}>
        {props.children}
      </touchArea>
    </focusScope>
    {opened ? <popupWindow nativeKey={popupKey} anchor={props.id} placement={props.placement ?? 'top'}
      width={width} window_layer="popover" dismiss_policy="outside_pointer_or_escape"
      containment="none" restore_focus={false} role="tooltip"
      accessible_name={props.description} onDismiss={activity.dismiss}>
      <touchArea nativeKey={`${popupKey}-hover`} enabled={true} mouse_cursor="default"
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
