/** @jsxImportSource @argui/react */
import { useId, useState, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import { Button } from './button'
import { ButtonGroupBoundary } from './button-group'
import type { WidgetTheme } from '../shared/theme'
import type { PopoverOptions } from '../shared/types'

/** Props for the native React Popover. */
export type PopoverProps = PopoverOptions & { children: ReactNode; leading?: ReactNode; trailing?: ReactNode }

/** Anchors arbitrary content to a simple trigger button and supports native dismissal. */
export function Popover(props: PopoverProps): ReactElement {
  const generatedId = `argui-popover-${useId().replaceAll(':', '')}`
  const [id] = useState(() => props.id ?? generatedId)
  const popupId = `${id}-popup`
  const theme = useTheme<WidgetTheme>()
  const [localOpen, setLocalOpen] = useState(props.defaultOpen ?? false)
  const expanded = props.open ?? localOpen
  const setOpen = (next: boolean) => {
    if (next === expanded) return
    if (props.open === undefined) setLocalOpen(next)
    props.onOpenChange?.(next)
  }

  return <column width={props.width} height={props.height}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}>
    <Button
      id={id}
      width={props.width}
      variant="secondary"
      expanded={expanded}
      controls={popupId}
      accessibleName={props.accessibleLabel ?? props.trigger}
      onClick={() => setOpen(!expanded)}
    >
      <row gap={theme.spacing} alignItems="center">
        {props.leading}<text color={theme.text}>{props.trigger}</text>{props.trailing}
      </row>
    </Button>
    {expanded ? <popupWindow
      id={popupId}
      anchor={id}
      placement={props.placement ?? 'bottomStart'}
      width={props.contentWidth ?? theme.overlayWidth}
      windowLayer="popover"
      dismissPolicy="outsidePointerOrEscape"
      containment="none"
      initialFocus={props.initialFocus === 'first' ? 'first' : ''}
      restoreFocus={true}
      accessibleName={props.accessibleLabel ?? props.trigger}
      onDismiss={() => setOpen(false)}
    >
      <rectangle
        width="100%"
        padding={theme.overlayPadding}
        background={props.opaque ? theme.surface : theme.overlaySurface}
        backdropFilter={!props.opaque && props.blur !== false ? `blur(${theme.overlayBlur}px)` : undefined}
        border={{ width: theme.overlayBorderWidth, color: theme.border }}
        radii={theme.overlayRadius}
        shadow={{ offsetY: theme.overlayShadowOffsetY, blur: theme.overlayShadowBlur, color: theme.overlayShadowColor }}
      >
        <column width="100%" gap={theme.spacing}>
          <ButtonGroupBoundary>
            {props.children}
            {props.closeLabel ? <Button
              variant="ghost"
              onClick={() => setOpen(false)}
            >{props.closeLabel}</Button> : null}
          </ButtonGroupBoundary>
        </column>
      </rectangle>
    </popupWindow> : null}
  </column>
}
