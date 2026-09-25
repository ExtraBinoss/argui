import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { PopoverProps as SharedPopoverProps } from '../shared/types'
import { Button } from './button'

export type PopoverProps = SharedPopoverProps<JSX.Element>

/** Presents arbitrary content in a themed, anchored native popover. */
export function Popover(props: PopoverProps): JSX.Element {
  const [localExpanded, setLocalExpanded] = createSignal(false)
  const expanded = () => props.open ?? localExpanded()
  const setExpanded = (open: boolean) => {
    if (props.onOpenChange) props.onOpenChange(open)
    else setLocalExpanded(open)
  }
  return <column>
    <Button id={props.id} label={props.label} theme={props.theme} kind="secondary"
      expanded={expanded()} controls={`${props.id}-popup`} onClick={() => setExpanded(!expanded())} />
    {expanded() ? <popupWindow key={`${props.id}-popup`} anchor={props.id} placement={props.placement ?? 'bottom_start'} width={props.width ?? 260}
      window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap"
      initial_focus="first" restore_focus={true} onDismiss={() => setExpanded(false)}>
      <rectangle width={props.width ?? 260} background={props.opaque ? props.theme.surface : props.theme.overlaySurface}
        backdrop_filter={props.opaque || props.blur === false ? undefined : 'blur(10px)'}
        border_color={props.theme.border} border_width={1} radius={props.theme.overlayRadius}
        shadow_blur={props.theme.overlayShadowBlur} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={12} padding={props.theme.overlayPadding}>
          {props.children}
          {props.closeLabel !== false ? <Button id={`${props.id}-close`} label={props.closeLabel ?? 'Close'} theme={props.theme} kind="outline"
            onClick={() => setExpanded(false)} /> : null}
        </column>
      </rectangle>
    </popupWindow> : null}
  </column>
}
