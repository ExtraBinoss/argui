import { createSignal, createUniqueId } from 'solid-js'
import { boundedWindow, nativeWindowRange, type NativeWindowRange, type VirtualListOptions } from '@argui/host'
import type { JSX } from './jsx-runtime'

/** Props for a native list that mounts only its visible keyed items. */
export interface VirtualListProps extends VirtualListOptions {
  id?: string
  renderItem: (index: number) => JSX.Element
}

/** Renders only ranges requested by the native virtual list as its viewport moves. */
export function VirtualList(props: VirtualListProps): JSX.Element {
  const generatedId = `virtual-list-${createUniqueId()}`
  const id = () => props.id ?? generatedId
  const [window, setWindow] = createSignal<NativeWindowRange>({
    start: 0, end: Math.min(props.count, 12), offset: 0, viewportExtent: 0,
  })
  const axis = () => props.axis ?? 'vertical'
  const shadow = () => props.shadow
  const current = () => boundedWindow(window(), props.count)
  const rows = () => {
    const range = current()
    return Array.from({ length: range.end - range.start }, (_, position) => {
      const index = range.start + position
      return <container key={`${id()}-item-${props.itemKey?.(index) ?? index}`} shrink={0}>{props.renderItem(index)}</container>
    })
  }
  return <virtualWindow
    key={id()}
    width={props.width ?? (axis() === 'horizontal' ? 'fill' : undefined)}
    height={props.height ?? (axis() === 'vertical' ? 'fill' : undefined)}
    horizontal={axis() === 'horizontal'}
    row_height={props.estimate ?? 48}
    variable_height={props.variable ?? true}
    viewport_width={axis() === 'horizontal' && current().viewportExtent > 0 ? current().viewportExtent : undefined}
    viewport_height={axis() === 'vertical' && current().viewportExtent > 0 ? current().viewportExtent : undefined}
    offset={current().offset}
    overscan={props.overscan ?? 3}
    scrollbar_visible={props.scrollbarVisible ?? !!props.scrollbarColor}
    scrollbar_width={props.scrollbarWidth}
    scrollbar_thumb={props.scrollbarColor}
    shadow_color={shadow()?.color}
    shadow_intensity={shadow()?.intensity}
    shadow_width={shadow()?.width}
    shadow_start={axis() === 'horizontal' ? shadow()?.left : shadow()?.top}
    shadow_end={axis() === 'horizontal' ? shadow()?.right : shadow()?.bottom}
    __item_count={props.count}
    __data_version={props.dataVersion ?? 0}
    __window_start={current().start}
    onWindow={(payload: unknown) => {
      const next = nativeWindowRange(payload, props.count)
      if (next && (next.start !== window().start || next.end !== window().end
        || next.viewportExtent !== window().viewportExtent)) setWindow(next)
    }}
  >{rows()}</virtualWindow>
}
