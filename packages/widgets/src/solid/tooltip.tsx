import { createSignal, createUniqueId } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { useTheme } from '@argui/solid'
import type { WidgetTheme } from '../shared/theme'
import type { TooltipOptions } from '../shared/types'

/** Props for the native Solid Tooltip. Children form its hover trigger. */
export type TooltipProps = TooltipOptions & { children: JSX.Element }

/** Shows short information beside a hovered or focused trigger. */
export function Tooltip(props: TooltipProps): JSX.Element {
  const id = props.id ?? `argui-tooltip-${createUniqueId()}`
  const popupId = `${id}-popup`
  const theme = useTheme<WidgetTheme>()
  const [open, setOpen] = createSignal(false)

  return <focusScope width={props.width} height={props.height}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}
    focusable={true} focusOnTabNavigation={true} accessibleDescription={props.content} describedBy={popupId}
    onFocus={() => setOpen(true)} onBlur={() => setOpen(false)}>
    <touchArea id={id} onPointerEnter={() => setOpen(true)}>{props.children}</touchArea>
    {open() ? <popupWindow id={popupId} anchor={id} role="tooltip" accessibleName={props.content}
      placement={props.placement ?? 'top'} width={props.contentWidth ?? 220}
      windowLayer="popover" dismissPolicy="outsideHoverOrEscape"
      containment="none" initialFocus="" restoreFocus={false}
      allowOutsideWindow={props.allowOutsideWindow} onDismiss={() => setOpen(false)}>
      <rectangle padding={{ top: 7, bottom: 7, start: 10, end: 10 }}
        background={props.opaque ? theme().surface : theme().overlaySurface}
        backdropFilter={!props.opaque && props.blur !== false ? `blur(${theme().overlayBlur}px)` : undefined}
        border={{ width: theme().overlayBorderWidth, color: theme().border }}
        radii={theme().overlayRadius}
        shadow={props.allowOutsideWindow ? undefined : { offsetY: theme().overlayShadowOffsetY, blur: theme().overlayShadowBlur, color: theme().overlayShadowColor }}>
        <text color={theme().text} fontSize={14} weight={500}>{props.content}</text>
      </rectangle>
    </popupWindow> : null}
  </focusScope>
}
