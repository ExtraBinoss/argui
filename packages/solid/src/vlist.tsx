import { For, createMemo, createSignal, untrack, type Accessor } from 'solid-js'
import { boundedWindow, nativeWindowRange, type NativeWindowRange, type VirtualListOptions } from '@argui/host'
import type { JSX } from './jsx-runtime'

/** Props for a native list that mounts only its visible keyed items. */
export type VirtualListProps = VirtualListOptions & {
  id?: string
  renderItem: (index: number) => JSX.Element
}

/** Renders only ranges requested by the native virtual list as its viewport moves. */
export function VirtualList(props: VirtualListProps): JSX.Element {
  const initial = Math.max(0, Math.min(props.count - 1,
    Number.isFinite(props.initialIndex) ? Math.trunc(props.initialIndex!) : 0))
  const [window, setWindow] = createSignal<NativeWindowRange>({
    start: initial, end: Math.min(props.count, initial + 12), offset: initial * (props.estimate ?? 48), viewportExtent: 0,
  })
  const axis = () => props.axis ?? 'vertical'
  const shadow = () => props.shadow
  const current = () => props.count <= 32
    ? { ...boundedWindow(window(), props.count), start: 0, end: props.count }
    : boundedWindow(window(), props.count)
  const entries = new Map<string | number, { index: Accessor<number>; setIndex: (index: number) => void }>()
  const rows = createMemo(() => {
    const range = current()
    const seen = new Set<string | number>()
    const next = Array.from({ length: range.end - range.start }, (_, position) => {
      const index = range.start + position
      const key = props.itemKey(index)
      if (seen.has(key)) throw new Error(`Duplicate virtual list item key: ${key}`)
      seen.add(key)
      let entry = entries.get(key)
      if (!entry) {
        const [read, write] = createSignal(index)
        entry = { index: read, setIndex: write }
        entries.set(key, entry)
      } else if (untrack(entry.index) !== index) entry.setIndex(index)
      return entry
    })
    for (const key of entries.keys()) if (!seen.has(key)) entries.delete(key)
    return next
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
    horizontal={axis() === 'horizontal'}
    rowHeight={props.estimate ?? 48}
    variableHeight={props.variable ?? true}
    viewportWidth={axis() === 'horizontal' && current().viewportExtent > 0 ? current().viewportExtent : undefined}
    viewportHeight={axis() === 'vertical' && current().viewportExtent > 0 ? current().viewportExtent : undefined}
    offset={current().offset}
    overscan={props.overscan ?? 3}
    scrollbarVisible={props.scrollbarVisible ?? !!props.scrollbarColor}
    scrollbarWidth={props.scrollbarWidth}
    scrollbarThumb={props.scrollbarColor}
    shadowColor={shadow()?.color}
    shadowIntensity={shadow()?.intensity}
    shadowWidth={shadow()?.width}
    shadowStart={axis() === 'horizontal' ? shadow()?.left : shadow()?.top}
    shadowEnd={axis() === 'horizontal' ? shadow()?.right : shadow()?.bottom}
    itemCount={props.count}
    dataVersion={props.dataVersion ?? 0}
    windowStart={current().start}
    onWindow={(payload) => {
      const next = nativeWindowRange(payload, props.count)
      if (next) {
        if (next.start !== window().start || next.end !== window().end
          || next.viewportExtent !== window().viewportExtent) setWindow(next)
        props.onWindowChange?.(next)
      }
    }}
    onMeasure={props.onMeasure}
  ><For each={rows()}>{(entry) => <container shrink={0}>{props.renderItem(entry.index())}</container>}</For></virtualWindow>
}
