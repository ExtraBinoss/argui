/** Per-edge controls for a native feathered scroll shadow. */
export interface ScrollShadowOptions {
  /** Optional tint; omit to fade content into the underlying backdrop. */
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
interface VirtualListBaseOptions {
  /** Minimum width on the list's native root. */
  minWidth?: ConstraintValue
  /** Maximum width on the list's native root. */
  maxWidth?: ConstraintValue
  /** Minimum height on the list's native root. */
  minHeight?: ConstraintValue
  /** Maximum height on the list's native root. */
  maxHeight?: ConstraintValue
  /** How much the list gives up when space is limited. */
  shrink?: number
  /** Alignment of the list within its parent layout. */
  alignSelf?: 'start' | 'center' | 'end' | 'stretch'
  /** Outer spacing on the native list root. */
  margin?: InsetsValue
  /** Logical item count; only a bounded native-requested range is rendered. */
  count: number
  /** Row placed at the top when the virtual viewport first mounts. */
  initialIndex?: number
  /** Observe native range changes without replacing the list's internal presenter. */
  onWindowChange?: (range: NativeWindowRange) => void
  /** Observe native item measurements, including the viewport extent. */
  onMeasure?: (payload: NativeEventPayload<'measure'>) => void
  /** Stable item identity across inserts and reordering, independent of the native node ID. */
  itemKey: (index: number) => string | number
  /** Increment after middle inserts, removals, or reorder to reset native size measurements. */
  dataVersion?: number
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
}

/** A virtual viewport must have a resolvable size on its scroll axis. */
export type VirtualListOptions = VirtualListBaseOptions & (
  | { axis?: 'vertical'; width?: DimensionValue; height: DimensionValue; grow?: number }
  | { axis?: 'vertical'; width?: DimensionValue; height?: DimensionValue; grow: number }
  | { axis: 'horizontal'; width: DimensionValue; height?: DimensionValue; grow?: number }
  | { axis: 'horizontal'; width?: DimensionValue; height?: DimensionValue; grow: number }
)

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
import type { ConstraintValue, DimensionValue, InsetsValue } from './protocol'
import type { NativeEventPayload } from './events'
