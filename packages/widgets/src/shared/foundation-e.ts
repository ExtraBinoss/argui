import type { SemanticActionPayload } from '@argui/host'
import type { Palette } from './theme'

/** Props for a themed determinate or indeterminate progress indicator. */
export interface ProgressProps {
  theme: Palette
  value?: number | null
  max?: number
  label?: string
  width?: number | 'fill'
  height?: number
}

/** Values accepted by the Slider component, including multi-thumb ranges. */
export type SliderValues = number | readonly number[]

/** Finite slider bounds and a positive snapping interval. */
export interface SliderDomain {
  min: number
  max: number
  step: number
}

/** Props for a pointer, keyboard, and accessibility-operable slider. */
export interface SliderProps {
  id: string
  label: string
  theme: Palette
  value?: SliderValues
  defaultValue?: SliderValues
  onValueChange?: (values: number[]) => void
  min?: number
  max?: number
  step?: number
  disabled?: boolean
  orientation?: 'horizontal' | 'vertical'
  width?: number
  height?: number
}

/** Props for a controlled or uncontrolled two-state switch. */
export interface SwitchProps {
  id: string
  label: string
  theme: Palette
  checked?: boolean
  defaultChecked?: boolean
  onCheckedChange?: (checked: boolean) => void
  disabled?: boolean
  size?: 'sm' | 'default'
}

/** Produces a finite, ascending slider domain from optional public props. */
export function sliderDomain(minValue: number | undefined, maxValue: number | undefined,
  stepValue: number | undefined): SliderDomain {
  const min = Number.isFinite(minValue) ? minValue! : 0
  const max = Number.isFinite(maxValue) && maxValue! > min ? maxValue! : min + 100
  const step = Number.isFinite(stepValue) && stepValue! > 0 ? stepValue! : 1
  return { min, max, step }
}

/** Normalizes finite slider values to the supplied domain and step grid. */
export function normalizeSliderValues(
  input: SliderValues | undefined,
  min: number,
  max: number,
  step: number,
): number[] {
  const source = typeof input === 'number' ? [input]
    : Array.isArray(input) ? input : [min]
  const values = source.map((value) => {
    const finite = Number.isFinite(value) ? value : min
    const bounded = Math.max(min, Math.min(max, finite))
    if (bounded === min || bounded === max) return bounded
    return Math.max(min, Math.min(max, min + Math.round((bounded - min) / step) * step))
  })
  return values.length ? values.sort((left, right) => left - right) : [min]
}

/** Converts a native pointer event into a clamped slider fraction. */
export function sliderPointerRatio(
  payload: unknown,
  orientation: 'horizontal' | 'vertical',
  inset: number,
): number | undefined {
  if (typeof payload !== 'object' || payload === null) return undefined
  const pointer = payload as Record<string, unknown>
  const position = pointer[orientation === 'horizontal' ? 'localX' : 'localY']
  const extent = pointer[orientation === 'horizontal' ? 'width' : 'height']
  if (typeof position !== 'number' || !Number.isFinite(position)
    || typeof extent !== 'number' || !Number.isFinite(extent) || extent <= inset * 2) return undefined
  const fraction = Math.max(0, Math.min(1, (position - inset) / (extent - inset * 2)))
  return orientation === 'vertical' ? 1 - fraction : fraction
}

/** Snaps one thumb's next value without allowing it to cross adjacent thumbs. */
export function adjustSliderThumb(values: readonly number[], index: number, nextValue: number,
  min: number, max: number, step: number): number[] {
  if (!Number.isFinite(nextValue) || index < 0 || index >= values.length) return [...values]
  const candidate = normalizeSliderValues(nextValue, min, max, step)[0] ?? min
  const lower = index > 0 ? values[index - 1]! : min
  const upper = index + 1 < values.length ? values[index + 1]! : max
  const adjusted = [...values]
  adjusted[index] = Math.max(lower, Math.min(upper, candidate))
  return adjusted
}

/** Computes one thumb's value after an accessibility semantic action. */
export function sliderActionValue(
  payload: SemanticActionPayload,
  current: number,
  min: number,
  max: number,
  step: number,
): number | undefined {
  if (payload.action === 'increment') return Math.min(max, current + step)
  if (payload.action === 'decrement') return Math.max(min, current - step)
  if (payload.action === 'set_value' && typeof payload.value === 'number') return payload.value
  return undefined
}

/** Maps slider keyboard input to its next bounded value. */
export function sliderKeyboardValue(
  payload: unknown,
  current: number,
  min: number,
  max: number,
  step: number,
  orientation: 'horizontal' | 'vertical',
): number | undefined {
  if (typeof payload !== 'object' || payload === null) return undefined
  const keyEvent = payload as Record<string, unknown>
  if (keyEvent.state !== undefined && keyEvent.state !== 'pressed') return undefined
  const key = typeof keyEvent.key === 'string' ? keyEvent.key : ''
  if (key === 'Home') return min
  if (key === 'End') return max
  if (key === 'PageUp') return Math.min(max, current + step * 10)
  if (key === 'PageDown') return Math.max(min, current - step * 10)
  if (key === 'ArrowRight' || (orientation === 'vertical' && key === 'ArrowUp')) {
    return Math.min(max, current + step)
  }
  if (key === 'ArrowLeft' || (orientation === 'vertical' && key === 'ArrowDown')) {
    return Math.max(min, current - step)
  }
  if (orientation === 'horizontal' && key === 'ArrowUp') return Math.min(max, current + step)
  if (orientation === 'horizontal' && key === 'ArrowDown') return Math.max(min, current - step)
  return undefined
}
