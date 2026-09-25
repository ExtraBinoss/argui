import type { Palette } from './theme'

/** Orientation used by the split-pane and slide controls. */
export type SurfaceEOrientation = 'horizontal' | 'vertical'

/** Common pointer, keyboard, and value props for a resizable split pane. */
export interface SurfaceEResizableProps<TContent> {
  /** Stable key prefix for the split pane and separator. */
  id: string
  /** Palette used for the panels, separator, and focus state. */
  theme: Palette
  /** Accessible name for the split-pane group. */
  label?: string
  /** Content displayed in the leading panel. */
  first: TContent
  /** Content displayed in the trailing panel. */
  second: TContent
  /** Panel flow direction. Defaults to horizontal. */
  orientation?: SurfaceEOrientation
  /** Overall width in logical pixels. Defaults to 640. */
  width?: number
  /** Overall height in logical pixels. Defaults to 260. */
  height?: number
  /** Controlled leading-panel percentage. Omit to use defaultValue. */
  value?: number
  /** Initial leading-panel percentage. Defaults to 50. */
  defaultValue?: number
  /** Minimum allowed leading-panel percentage. Defaults to 15. */
  minValue?: number
  /** Maximum allowed leading-panel percentage. Defaults to 85. */
  maxValue?: number
  /** Percentage change per keyboard adjustment. Defaults to 1. */
  step?: number
  /** Accessible name for the divider. Defaults to `Resize panels`. */
  handleLabel?: string
  /** Called when pointer, keyboard, or assistive input changes the split. */
  onValueChange?: (value: number) => void
}

/** Props for a single-date calendar with controlled or local state. */
export interface SurfaceECalendarProps {
  /** Stable key prefix for the calendar and its date cells. */
  id: string
  /** Palette used for the calendar surface, dates, and focus state. */
  theme: Palette
  /** Selected local calendar date. Omit to use defaultValue. */
  value?: Date
  /** Initial selected local calendar date. */
  defaultValue?: Date
  /** Controlled month displayed by the calendar. */
  month?: Date
  /** Initial displayed month. Defaults to the selected date or current month. */
  defaultMonth?: Date
  /** First weekday index, Sunday 0 through Saturday 6. Defaults to 0. */
  weekStartsOn?: number
  /** Locale passed to Intl.DateTimeFormat. Defaults to `en-US`. */
  locale?: string
  /** Whether adjacent-month dates remain visible and selectable. Defaults to true. */
  showOutsideDays?: boolean
  /** Optional predicate which disables a local calendar date. */
  isDateDisabled?: (date: Date) => boolean
  /** Inclusive earliest selectable local date. */
  minDate?: Date
  /** Inclusive latest selectable local date. */
  maxDate?: Date
  /** Accessible calendar name. Defaults to `Choose date`. */
  label?: string
  /** Called when the selected date changes. */
  onValueChange?: (value: Date) => void
  /** Called when the displayed month changes. */
  onMonthChange?: (month: Date) => void
}

/** Controlled or uncontrolled slide collection for a themed carousel. */
export interface SurfaceECarouselProps<TSlide> {
  /** Stable key prefix for the carousel and slide region. */
  id: string
  /** Palette used for slide surfaces, controls, and indicators. */
  theme: Palette
  /** Slides in navigation order. */
  slides: readonly TSlide[]
  /** Controlled zero-based active slide. Omit to use defaultIndex. */
  index?: number
  /** Initial zero-based slide. Defaults to zero. */
  defaultIndex?: number
  /** Whether next and previous navigation wraps at the collection ends. */
  loop?: boolean
  /** Horizontal or vertical navigation. Defaults to horizontal. */
  orientation?: SurfaceEOrientation
  /** Accessible region name. Defaults to `Carousel`. */
  label?: string
  /** Slide region height in logical pixels. Defaults to 220. */
  height?: number
  /** Called when keyboard, pointer, or button navigation changes the slide. */
  onValueChange?: (index: number) => void
}

/** Date shown in one calendar cell. */
export interface SurfaceECalendarCell {
  /** Date at local noon, preserving the calendar day across daylight changes. */
  date: Date
  /** Stable local date key used for native focus and list reconciliation. */
  key: string
  /** Whether the date belongs to the displayed month. */
  inMonth: boolean
}

/** Reads a pressed key name from the native host key event. */
export function surfaceEKey(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (typeof payload !== 'object' || payload === null || !('key' in payload)) return undefined
  const event = payload as { key?: unknown; state?: unknown }
  if (event.state !== undefined && event.state !== 'pressed') return undefined
  return typeof event.key === 'string' ? event.key : undefined
}

/** Returns a stable YYYY-MM-DD key for the local calendar date. */
export function surfaceEDateKey(date: Date): string {
  const year = date.getFullYear().toString().padStart(4, '0')
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const day = date.getDate().toString().padStart(2, '0')
  return `${year}-${month}-${day}`
}

/** Creates a local-noon Date from its stable YYYY-MM-DD calendar key. */
export function surfaceEParseDateKey(key: string): Date | undefined {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(key)
  if (!match) return undefined
  const year = Number(match[1])
  const month = Number(match[2]) - 1
  const day = Number(match[3])
  const date = new Date(year, month, day, 12)
  return date.getFullYear() === year && date.getMonth() === month && date.getDate() === day
    ? date : undefined
}

/** Returns a local-noon copy of a date, or undefined for invalid input. */
export function surfaceEValidDate(value: Date | undefined): Date | undefined {
  if (!(value instanceof Date) || !Number.isFinite(value.getTime())) return undefined
  return new Date(value.getFullYear(), value.getMonth(), value.getDate(), 12)
}

/** Moves one date by calendar days without relying on a fixed day duration. */
export function surfaceEAddDays(date: Date, amount: number): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() + amount, 12)
}

/** Moves one date by calendar months while keeping its day where possible. */
export function surfaceEAddMonths(date: Date, amount: number): Date {
  const targetMonth = new Date(date.getFullYear(), date.getMonth() + amount, 1, 12)
  const lastDay = new Date(targetMonth.getFullYear(), targetMonth.getMonth() + 1, 0, 12).getDate()
  return new Date(targetMonth.getFullYear(), targetMonth.getMonth(), Math.min(date.getDate(), lastDay), 12)
}

/** Builds six calendar weeks, ordered from the configured first weekday. */
export function surfaceECalendarCells(month: Date, weekStartsOn: number): SurfaceECalendarCell[] {
  const first = new Date(month.getFullYear(), month.getMonth(), 1, 12)
  const offset = (first.getDay() - normalizeWeekday(weekStartsOn) + 7) % 7
  const start = surfaceEAddDays(first, -offset)
  return Array.from({ length: 42 }, (_, index) => {
    const date = surfaceEAddDays(start, index)
    return { date, key: surfaceEDateKey(date), inMonth: date.getMonth() === month.getMonth() }
  })
}

/** Maps calendar arrow and paging keys to the next active date. */
export function surfaceECalendarKeyDate(
  date: Date,
  key: string,
  weekStartsOn: number,
): Date | undefined {
  if (key === 'ArrowLeft') return surfaceEAddDays(date, -1)
  if (key === 'ArrowRight') return surfaceEAddDays(date, 1)
  if (key === 'ArrowUp') return surfaceEAddDays(date, -7)
  if (key === 'ArrowDown') return surfaceEAddDays(date, 7)
  if (key === 'Home') {
    return surfaceEAddDays(date, -((date.getDay() - normalizeWeekday(weekStartsOn) + 7) % 7))
  }
  if (key === 'End') {
    return surfaceEAddDays(date, (normalizeWeekday(weekStartsOn) + 6 - date.getDay() + 7) % 7)
  }
  if (key === 'PageUp') return surfaceEAddMonths(date, -1)
  if (key === 'PageDown') return surfaceEAddMonths(date, 1)
  return undefined
}

/** Returns whether a calendar date is contained by optional inclusive bounds. */
export function surfaceEWithinCalendarBounds(
  date: Date,
  minDate: Date | undefined,
  maxDate: Date | undefined,
): boolean {
  const key = surfaceEDateKey(date)
  return (!minDate || key >= surfaceEDateKey(minDate)) && (!maxDate || key <= surfaceEDateKey(maxDate))
}

/** Clamps a finite split percentage to the supplied inclusive bounds. */
export function surfaceEClampSplit(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, Number.isFinite(value) ? value : min))
}

/** Converts one drag displacement into an updated split percentage. */
export function surfaceEResizeByPixels(
  value: number,
  delta: unknown,
  extent: number,
  min: number,
  max: number,
): number {
  return typeof delta === 'number' && Number.isFinite(delta) && Number.isFinite(extent) && extent > 0
    ? surfaceEClampSplit(value + delta / extent * 100, min, max)
    : value
}

/** Extracts a finite logical-pixel displacement from a native drag callback. */
export function surfaceEDragPixels(payload: unknown): number {
  return typeof payload === 'number' && Number.isFinite(payload) ? payload : 0
}

/** Advances a slide index while respecting a bounded or wrapping carousel. */
export function surfaceECarouselIndex(
  current: number,
  direction: -1 | 1,
  count: number,
  loop: boolean,
): number {
  if (count <= 0) return 0
  const next = current + direction
  return loop ? (next + count) % count : Math.max(0, Math.min(count - 1, next))
}

function normalizeWeekday(value: number): number {
  return Number.isFinite(value) ? ((Math.trunc(value) % 7) + 7) % 7 : 0
}
