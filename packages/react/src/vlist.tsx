/** @jsxImportSource @argui/react */
import { useState, type ReactNode, type ReactElement } from 'react'
import { boundedWindow, nativeWindowRange, type NativeWindowRange, type VirtualListOptions } from '@argui/host'

/** Props for a native list that mounts only its visible keyed items. */
export type VirtualListProps = VirtualListOptions & {
  id?: string
  renderItem: (index: number) => ReactNode
}

/** Renders only ranges requested by the native virtual list as its viewport moves. */
export function VirtualList(props: VirtualListProps): ReactElement {
  const initial = Math.max(0, Math.min(props.count - 1,
    Number.isFinite(props.initialIndex) ? Math.trunc(props.initialIndex!) : 0))
  const [window, setWindow] = useState<NativeWindowRange>({
    start: initial, end: Math.min(props.count, initial + 12), offset: initial * (props.estimate ?? 48), viewportExtent: 0,
  })
  const current = props.count <= 32
    ? { ...boundedWindow(window, props.count), start: 0, end: props.count }
    : boundedWindow(window, props.count)
  const axis = props.axis ?? 'vertical'
  const seen = new Set<string | number>()
  const rows = Array.from({ length: current.end - current.start }, (_, position) => {
    const index = current.start + position
    const key = props.itemKey(index)
    if (seen.has(key)) throw new Error(`Duplicate virtual list item key: ${key}`)
    seen.add(key)
    return <container key={key} shrink={0}>{props.renderItem(index)}</container>
  })
  return <virtualWindow
    id={props.id}
    width={props.width}
    height={props.height}
    minWidth={props.minWidth}
    maxWidth={props.maxWidth}
    minHeight={props.minHeight}
    maxHeight={props.maxHeight}
    grow={props.grow}
    shrink={props.shrink}
    alignSelf={props.alignSelf}
    margin={props.margin}
    horizontal={axis === 'horizontal'}
    rowHeight={props.estimate ?? 48}
    variableHeight={props.variable ?? true}
    viewportWidth={axis === 'horizontal' && current.viewportExtent > 0 ? current.viewportExtent : undefined}
    viewportHeight={axis === 'vertical' && current.viewportExtent > 0 ? current.viewportExtent : undefined}
    offset={current.offset}
    overscan={props.overscan ?? 3}
    scrollbarVisible={props.scrollbarVisible ?? !!props.scrollbarColor}
    scrollbarWidth={props.scrollbarWidth}
    scrollbarThumb={props.scrollbarColor}
    shadowColor={props.shadow?.color}
    shadowIntensity={props.shadow?.intensity}
    shadowWidth={props.shadow?.width}
    shadowStart={axis === 'horizontal' ? props.shadow?.left : props.shadow?.top}
    shadowEnd={axis === 'horizontal' ? props.shadow?.right : props.shadow?.bottom}
    itemCount={props.count}
    dataVersion={props.dataVersion ?? 0}
    windowStart={current.start}
    onWindow={(payload) => {
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
