import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { surfaceDragDelta, surfacePanelBounds, type SurfacePanelProps, type SurfacePanelSide } from '../shared/surface-d'
import { Button } from './button'

export type DrawerProps = SurfacePanelProps<JSX.Element>

/** Opens a modal panel with a touch handle that can swipe-dismiss the drawer. */
export function Drawer(props: DrawerProps): JSX.Element {
  const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false)
  const opened = () => props.open ?? localOpen()
  const requestOpen = (value: boolean) => {
    if (opened() === value) return
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
  let dragDistance = 0
  const onDragX = (payload: unknown) => {
    if (!horizontal) return
    dragDistance += surfaceDragDelta(payload)
    if ((side === 'right' && dragDistance > 72) || (side === 'left' && dragDistance < -72)) {
      dragDistance = 0
      requestOpen(false)
    }
  }
  const onDragY = (payload: unknown) => {
    if (horizontal) return
    dragDistance += surfaceDragDelta(payload)
    if ((side === 'bottom' && dragDistance > 72) || (side === 'top' && dragDistance < -72)) {
      dragDistance = 0
      requestOpen(false)
    }
  }
  const handle = horizontal
    ? <row width="fill" height={56} align_items="center" justify_content="center">
      <touchArea key={`${popupKey}-handle`} width={28} height={52} accessible_hidden={true}
        onPointerDown={() => { dragDistance = 0 }} onPointerUp={() => { dragDistance = 0 }}
        onPointerCancel={() => { dragDistance = 0 }} onDragX={onDragX}>
        <rectangle width={5} height={44} radius={3} background={props.theme.border} />
      </touchArea>
    </row>
    : <row width="fill" height={20} align_items="center" justify_content="center">
      <touchArea key={`${popupKey}-handle`} width={56} height={20} accessible_hidden={true}
        onPointerDown={() => { dragDistance = 0 }} onPointerUp={() => { dragDistance = 0 }}
        onPointerCancel={() => { dragDistance = 0 }} onDragY={onDragY}>
        <rectangle width={44} height={5} radius={3} background={props.theme.border} />
      </touchArea>
    </row>
  return <>
    <Button id={`${props.id}-trigger`} label={props.triggerLabel} theme={props.theme}
      kind="outline" expanded={opened()} controls={popupKey} onClick={() => requestOpen(true)} />
    {opened() ? <popupWindow key={popupKey} placement="fill" width="fill" height="fill"
      window_layer="modal" containment="modal" dismiss_policy="outside_pointer_or_escape"
      initial_focus={closeLabel === false ? 'first' : closeKey} restore_focus={true}
      role="dialog" modal={true} accessible_name={props.title} accessible_description={props.description}
      onDismiss={() => requestOpen(false)}>
      <container width="fill" height="fill" position="relative">
        <touchArea key={`${popupKey}-backdrop`} width="fill" height="fill" onClick={() => requestOpen(false)}>
          <rectangle width="fill" height="fill" background="#090c1c99" backdrop_filter="blur(10px)" />
        </touchArea>
        <container position="absolute" z_index={1} width={bounds.width} height={bounds.height}
          inset_left={bounds.insetLeft} inset_right={bounds.insetRight}
          inset_top={bounds.insetTop} inset_bottom={bounds.insetBottom}>
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
