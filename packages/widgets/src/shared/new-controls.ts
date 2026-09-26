import type { WidgetLayoutProps } from './types'

/** Controlled, read-only, or local boolean state shared by Checkbox and Switch. */
export type BooleanControlOptions = Omit<WidgetLayoutProps, 'height' | 'minHeight' | 'maxHeight'> & {
  accessibleName: string
  label?: string
  disabled?: boolean
  description?: string
} & (
  | { value: boolean; onValueChange: (value: boolean) => void; defaultValue?: never }
  | { value: boolean; onValueChange?: never; defaultValue?: never }
  | { value?: never; defaultValue?: boolean; onValueChange?: (value: boolean) => void }
)

/** One tab and its panel content, with a stable value. */
export interface TabItem<Content> {
  value: string
  label: string
  content: Content
  disabled?: boolean
}

/** Shared configuration for a single-selection tab set. */
export type TabsOptions<Content> = WidgetLayoutProps & {
  accessibleName: string
  items: readonly TabItem<Content>[]
  variant?: 'default' | 'line'
  orientation?: 'horizontal' | 'vertical'
} & (
  | { value: string; onValueChange: (value: string) => void; defaultValue?: never }
  | { value: string; onValueChange?: never; defaultValue?: never }
  | { value?: never; defaultValue?: string; onValueChange?: (value: string) => void }
)

/** Slider range with local or controlled ownership. */
export type SliderOptions = WidgetLayoutProps & {
  accessibleName: string
  min?: number
  max?: number
  step?: number
  disabled?: boolean
} & (
  | { value: number; onValueChange: (value: number) => void; defaultValue?: never }
  | { value: number; onValueChange?: never; defaultValue?: never }
  | { value?: never; defaultValue?: number; onValueChange?: (value: number) => void }
)

/** Read-only progress value; null announces an indeterminate operation. */
export type ProgressOptions = WidgetLayoutProps & {
  accessibleName: string
  value: number | null
  max?: number
  /** Pauses the native indeterminate sweep while preserving its current phase. */
  playing?: boolean
}

/** Returns a finite value clamped to the supplied range and snapped to step. */
export function snapValue(value: number, min: number, max: number, step: number): number {
  if (!Number.isFinite(min) || !Number.isFinite(max) || max <= min) return min
  const clamped = Math.max(min, Math.min(max, Number.isFinite(value) ? value : min))
  const increment = Number.isFinite(step) && step > 0 ? step : 1
  const snapped = min + Math.round((clamped - min) / increment) * increment
  return Math.max(min, Math.min(max, Number(snapped.toFixed(8))))
}

/** Maps a pointer position on a horizontal slider to a snapped value. */
export function sliderPointerValue(localX: number | undefined, width: number | undefined,
  min: number, max: number, step: number): number | undefined {
  if (localX === undefined || width === undefined || !Number.isFinite(localX) || width <= 16) return undefined
  const fraction = Math.max(0, Math.min(1, (localX - 8) / (width - 16)))
  return snapValue(min + fraction * (max - min), min, max, step)
}

/** Finds an enabled tab in the requested direction, wrapping at the ends. */
export function nextEnabledTab<Content>(items: readonly TabItem<Content>[], current: number,
  direction: 1 | -1): number {
  for (let step = 1; step <= items.length; step++) {
    const index = (current + direction * step + items.length) % items.length
    if (!items[index]?.disabled) return index
  }
  return -1
}
