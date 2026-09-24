/** Per-edge controls for a native feathered scroll shadow. */
export interface ScrollShadowOptions {
  /** CSS color used for the native shadow, including optional alpha. */
  color?: string
  /** Strength from zero to one; native geometry fades it at the scroll ends. */
  intensity?: number
  /** Feather width in logical pixels. */
  width?: number
  /** Whether to draw the leading horizontal edge. */
  left?: boolean
  /** Whether to draw the leading vertical edge. */
  top?: boolean
  /** Whether to draw the trailing horizontal edge. */
  right?: boolean
  /** Whether to draw the trailing vertical edge. */
  bottom?: boolean
}

/** Public options shared by Solid and React native virtual lists. */
export interface VirtualListOptions {
  /** Logical item count; only a bounded native-requested range is rendered. */
  count: number
  /** Stable item identity across inserts and reordering; defaults to its index. */
  itemKey?: (index: number) => string | number
  /** Increment after middle inserts, removals, or reorder to reset native size measurements. */
  dataVersion?: number
  /** Scroll direction; vertical by default. */
  axis?: 'horizontal' | 'vertical'
  /** Initial extent estimate before the native layout measures an item. */
  estimate?: number
  /** Measure each visible item's actual width or height; enabled by default. */
  variable?: boolean
  /** Extra rows retained around the visible range. */
  overscan?: number
  /** Show a native scrollbar; showing a colored thumb also enables it by default. */
  scrollbarVisible?: boolean
  /** Native scrollbar thickness in logical pixels. */
  scrollbarWidth?: number
  /** Native scrollbar thumb color. */
  scrollbarColor?: string
  /** Native shadow appearance and active edges. */
  shadow?: ScrollShadowOptions
  /** Cross-axis or container width; defaults to natural layout. */
  width?: number | string
  /** Container height; defaults to natural layout. */
  height?: number | string
}

/** A native-computed mounted range; JavaScript only renders its requested rows. */
export interface NativeWindowRange {
  start: number
  end: number
  offset: number
  viewportExtent: number
}

/** Decodes a native window change without calculating ranges in JavaScript. */
export function nativeWindowRange(payload: unknown, count: number): NativeWindowRange | undefined {
  if (!payload || typeof payload !== 'object') return undefined
  const value = payload as Record<string, unknown>
  const { start, end, offset, viewportExtent } = value
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end)
    || (start as number) < 0 || (end as number) < (start as number) || (end as number) > count
    || typeof offset !== 'number' || !Number.isFinite(offset) || offset < 0
    || typeof viewportExtent !== 'number' || !Number.isFinite(viewportExtent) || viewportExtent < 0) return undefined
  return { start: start as number, end: end as number, offset, viewportExtent }
}

/** Keeps a pending native range valid if the data set shrinks before its next update. */
export function boundedWindow(range: NativeWindowRange, count: number): NativeWindowRange {
  if (range.start < count) return { ...range, end: Math.min(range.end, count) }
  return { start: 0, end: Math.min(count, 12), offset: 0, viewportExtent: range.viewportExtent }
}
