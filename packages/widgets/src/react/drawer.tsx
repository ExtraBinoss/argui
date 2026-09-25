/** @jsxImportSource @argui/react */
import { useEffect, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { surfaceDragDistance, surfaceDragFinished, surfaceDragProjected, surfacePanelBounds, type SurfacePanelProps, type SurfacePanelSide } from '../shared/surface-d'
import { ReactButton as Button } from './button'

export type ReactDrawerProps = SurfacePanelProps<ReactNode>

/** Opens a modal panel with a touch handle that swipe-dismisses the drawer. */
export function ReactDrawer(props: ReactDrawerProps): ReactElement {
  const [localOpen, setLocalOpen] = useState(props.defaultOpen ?? false)
  const currentOpen = useRef(props.open ?? localOpen)
  currentOpen.current = props.open ?? localOpen
  const opened = props.open ?? localOpen
  const requestOpen = (value: boolean) => {
    if (currentOpen.current === value) return
    currentOpen.current = value
    if (props.open === undefined) setLocalOpen(value)
    props.onOpenChange?.(value)
  }
  const side: SurfacePanelSide = props.side ?? 'bottom'
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 420
  const height = Number.isFinite(props.height) && (props.height ?? 0) > 0 ? props.height! : 360
  const padding = Number.isFinite(props.padding) && (props.padding ?? -1) >= 0
    ? props.padding! : props.theme.overlayPadding
  const closeLabel = props.closeLabel === false ? false : props.closeLabel ?? 'Close'
  const closeKey = `${props.id}-close`
  const popupKey = `${props.id}-drawer`
  const bounds = surfacePanelBounds(side, width, height)
  const horizontal = side === 'left' || side === 'right'
  const exitOffset = (side === 'bottom' || side === 'right' ? 1 : -1) * ((horizontal ? width : height) + 40)
  const [present, setPresent] = useState(opened)
  const [handleHovered, setHandleHovered] = useState(false)
  const [dragOffset, setDragOffset] = useState(opened ? exitOffset : 0)
  const [dragging, setDragging] = useState(false)
  const dragDistance = useRef(0)
  useEffect(() => {
    setDragging(false)
    if (opened) {
      setDragOffset(exitOffset)
      setPresent(true)
      const timer = setTimeout(() => setDragOffset(0), 16)
      return () => clearTimeout(timer)
    }
    if (present) {
      setDragOffset(exitOffset)
      const timer = setTimeout(() => { setPresent(false); setDragOffset(0) }, 520)
      return () => clearTimeout(timer)
    }
  }, [opened, exitOffset])
  const resetDrag = () => { dragDistance.current = 0; setDragging(false); if (opened) setDragOffset(0) }
  const startDrag = () => { if (currentOpen.current) { dragDistance.current = 0; setDragging(true) } }
  const finishDrag = (payload: unknown, axis: 'x' | 'y') => {
    const outward = dragDistance.current * (exitOffset > 0 ? 1 : -1)
    const projected = surfaceDragProjected(payload, axis, dragDistance.current) * (exitOffset > 0 ? 1 : -1)
    if (outward >= 72 || (outward >= 20 && projected >= 72)) {
      setDragging(false)
      requestOpen(false)
    } else resetDrag()
  }
  const onDragX = (payload: unknown) => {
    if (!horizontal || !currentOpen.current) return
    dragDistance.current = surfaceDragDistance(payload, 'x', dragDistance.current)
    if (surfaceDragFinished(payload)) finishDrag(payload, 'x')
    else setDragOffset(side === 'right' ? Math.max(0, dragDistance.current) : Math.min(0, dragDistance.current))
  }
  const onDragY = (payload: unknown) => {
    if (horizontal || !currentOpen.current) return
    dragDistance.current = surfaceDragDistance(payload, 'y', dragDistance.current)
    if (surfaceDragFinished(payload)) finishDrag(payload, 'y')
    else setDragOffset(side === 'bottom' ? Math.max(0, dragDistance.current) : Math.min(0, dragDistance.current))
  }
  const handle = horizontal
    ? <row width="fill" height={56} align_items="center" justify_content="center">
      <touchArea nativeKey={`${popupKey}-handle`} width={28} height={52} accessible_hidden={true}
        mouse_cursor="grab" onPointerEnter={() => setHandleHovered(true)} onPointerLeave={() => setHandleHovered(false)}
        onPointerDown={startDrag} onPointerUp={() => setDragging(false)}
        onPointerCancel={resetDrag} onDragX={onDragX}>
        <rectangle width={5} height={44} radius={3}
          background={handleHovered ? props.theme.accent : props.theme.foreground} />
      </touchArea>
    </row>
    : <row width="fill" height={20} align_items="center" justify_content="center">
      <touchArea nativeKey={`${popupKey}-handle`} width={56} height={20} accessible_hidden={true}
        mouse_cursor="grab" onPointerEnter={() => setHandleHovered(true)} onPointerLeave={() => setHandleHovered(false)}
        onPointerDown={startDrag} onPointerUp={() => setDragging(false)}
        onPointerCancel={resetDrag} onDragY={onDragY}>
        <rectangle width={44} height={5} radius={3}
          background={handleHovered ? props.theme.accent : props.theme.foreground} />
      </touchArea>
    </row>
  return <>
    <Button id={`${props.id}-trigger`} label={props.triggerLabel} theme={props.theme}
      kind="outline" expanded={opened} controls={popupKey} onClick={() => requestOpen(true)} />
    {present ? <popupWindow nativeKey={popupKey} placement="fill" width="fill" height="fill"
      window_layer="modal" containment="modal" dismiss_policy="outside_pointer_or_escape"
      initial_focus={closeLabel === false ? 'first' : closeKey} restore_focus={true}
      role="dialog" modal={true} accessible_name={props.title} accessible_description={props.description}
      onDismiss={() => requestOpen(false)}>
      <container width="fill" height="fill" position="relative">
        <touchArea nativeKey={`${popupKey}-backdrop`} width="fill" height="fill"
          onClick={() => requestOpen(false)}>
          <rectangle width="fill" height="fill" background="#090c1c99" backdrop_filter="blur(10px)"
            opacity={opened ? 1 : 0} transition_ms={280} />
        </touchArea>
        <container position="absolute" z_index={1} width={bounds.width} height={bounds.height}
          inset_left={bounds.insetLeft} inset_right={bounds.insetRight}
          inset_top={bounds.insetTop} inset_bottom={bounds.insetBottom}
          transition_spring={!dragging}
          translate_x={horizontal ? dragOffset : 0} translate_y={horizontal ? 0 : dragOffset}>
          <rectangle width="fill" height="fill" background={props.theme.overlaySurface}
            border_color={props.theme.border} border_width={1} radius={props.theme.overlayRadius}
            backdrop_filter="blur(12px)" shadow_blur={props.theme.overlayShadowBlur}
            shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
            <column width="fill" height="fill" gap={16} padding={padding}>
              {side === 'top' ? null : handle}
              <row width="fill" gap={12} align_items="center" justify_content="space_between">
                <column width="fill" gap={5}>
                  <text width="fill" text={props.title} color={props.theme.foreground} font_size={18} weight={600} />
                  {props.description ? <text width="fill" text={props.description}
                    color={props.theme.muted} font_size={13} /> : null}
                </column>
                {closeLabel !== false ? <Button id={closeKey} label={closeLabel} theme={props.theme}
                  kind="ghost" onClick={() => requestOpen(false)} /> : null}
              </row>
              <column width="fill" grow={1} gap={12}>{props.content}</column>
              {side === 'top' ? handle : null}
            </column>
          </rectangle>
        </container>
      </container>
    </popupWindow> : null}
  </>
}
