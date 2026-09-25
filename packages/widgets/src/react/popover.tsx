/** @jsxImportSource @argui/react */
import { useState, type ReactElement, type ReactNode } from 'react'
import type { PopoverProps as SharedPopoverProps } from '../shared/types'
import { ReactButton } from './button'
import { VirtualList } from '@argui/react'

export type ReactPopoverProps = SharedPopoverProps<ReactNode>

/** Presents arbitrary content in the same anchored native popover as Solid. */
export function ReactPopover(props: ReactPopoverProps): ReactElement {
  const [localExpanded, setLocalExpanded] = useState(false)
  const expanded = props.open ?? localExpanded
  const setExpanded = (open: boolean) => {
    if (props.onOpenChange) props.onOpenChange(open)
    else setLocalExpanded(open)
  }
  const content = expanded ? <popupWindow nativeKey={`${props.id}-popup`} anchor={props.id} placement={props.placement ?? 'bottom_start'} width={props.width ?? 260}
      window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap"
      initial_focus={props.initialFocus ?? 'first'} restore_focus={true} onDismiss={() => setExpanded(false)}>
      <rectangle width={props.width ?? 260} background={props.opaque ? props.theme.surface : props.theme.overlaySurface}
        backdrop_filter={props.opaque || props.blur === false ? undefined : 'blur(10px)'}
        border_color={props.theme.border} border_width={1} radius={props.theme.overlayRadius}
        shadow_blur={props.theme.overlayShadowBlur} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={12} padding={props.contentPadding ?? props.theme.overlayPadding}>
          {props.virtualItems ? <focusScope role="list_box" accessible_name={props.label} focusable={false}>
            <VirtualList id={`${props.id}-items`} count={props.virtualItems.count}
              initialIndex={props.virtualItems.initialIndex} estimate={props.virtualItems.itemHeight}
              variable={false} height={props.virtualItems.height} width="fill" overscan={2}
              itemKey={props.virtualItems.itemKey} scrollbarVisible={true}
              renderItem={props.virtualItems.renderItem} />
          </focusScope> : props.children}
          {props.closeLabel !== false ? <ReactButton id={`${props.id}-close`} label={props.closeLabel ?? 'Close'} theme={props.theme} kind="outline"
            onClick={() => setExpanded(false)} /> : null}
        </column>
      </rectangle>
    </popupWindow> : null
  return props.trigger === false ? <>{content}</> : <column>
    <ReactButton id={props.id} label={props.label} theme={props.theme} kind="secondary"
      expanded={expanded} controls={`${props.id}-popup`} onClick={() => setExpanded(!expanded)} />
    {content}
  </column>
}
