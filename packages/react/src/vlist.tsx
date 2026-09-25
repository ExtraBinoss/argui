/** @jsxImportSource @argui/react */
import { useId, useState, type ReactNode, type ReactElement } from 'react'
import { boundedWindow, nativeWindowRange, type NativeWindowRange, type VirtualListOptions } from '@argui/host'

/** Props for a native list that mounts only its visible keyed items. */
export interface VirtualListProps extends VirtualListOptions {
  id?: string
  renderItem: (index: number) => ReactNode
}

/** Renders only ranges requested by the native virtual list as its viewport moves. */
export function VirtualList(props: VirtualListProps): ReactElement {
  const generatedId = useId()
  const id = props.id ?? `virtual-list-${generatedId}`
  const initial = Math.max(0, Math.min(props.count - 1,
    Number.isFinite(props.initialIndex) ? Math.trunc(props.initialIndex!) : 0))
  const [window, setWindow] = useState<NativeWindowRange>({
    start: initial, end: Math.min(props.count, initial + 12), offset: initial * (props.estimate ?? 48), viewportExtent: 0,
  })
  const current = props.count <= 32
    ? { ...boundedWindow(window, props.count), start: 0, end: props.count }
    : boundedWindow(window, props.count)
  const axis = props.axis ?? 'vertical'
  const rows = Array.from({ length: current.end - current.start }, (_, position) => {
    const index = current.start + position
    const key = props.itemKey?.(index) ?? index
    return <container key={key} nativeKey={`${id}-item-${key}`} shrink={0}>{props.renderItem(index)}</container>
  })
  return <virtualWindow
    nativeKey={id}
    width={props.width ?? (axis === 'horizontal' ? 'fill' : undefined)}
    height={props.height ?? (axis === 'vertical' ? 'fill' : undefined)}
    horizontal={axis === 'horizontal'}
    row_height={props.estimate ?? 48}
    variable_height={props.variable ?? true}
    viewport_width={axis === 'horizontal' && current.viewportExtent > 0 ? current.viewportExtent : undefined}
    viewport_height={axis === 'vertical' && current.viewportExtent > 0 ? current.viewportExtent : undefined}
    offset={current.offset}
    overscan={props.overscan ?? 3}
    scrollbar_visible={props.scrollbarVisible ?? !!props.scrollbarColor}
    scrollbar_width={props.scrollbarWidth}
    scrollbar_thumb={props.scrollbarColor}
    shadow_color={props.shadow?.color}
    shadow_intensity={props.shadow?.intensity}
    shadow_width={props.shadow?.width}
    shadow_start={axis === 'horizontal' ? props.shadow?.left : props.shadow?.top}
    shadow_end={axis === 'horizontal' ? props.shadow?.right : props.shadow?.bottom}
    __item_count={props.count}
    __data_version={props.dataVersion ?? 0}
    __window_start={current.start}
    onWindow={(payload: unknown) => {
      const next = nativeWindowRange(payload, props.count)
      if (next) {
        if (next.start !== window.start || next.end !== window.end
          || next.viewportExtent !== window.viewportExtent) setWindow(next)
        props.onWindowChange?.(next)
      }
    }}
    onMeasure={props.onMeasure}
  >{rows}</virtualWindow>
}
