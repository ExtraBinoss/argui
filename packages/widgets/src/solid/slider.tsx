import { createSignal } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { useTheme } from '@argui/solid'
import type { NativeEventPayload } from '@argui/host'
import type { WidgetTheme } from '../shared/theme'
import { sliderPointerValue, snapValue, type SliderOptions } from '../shared/new-controls'
import { keyName } from '../shared/types'

/** Props for a horizontal, single-thumb native slider. */
export type SliderProps = SliderOptions

/** Renders a step-aware slider with native pointer and keyboard interaction. */
export function Slider(props: SliderProps): JSX.Element {
  const theme = useTheme<WidgetTheme>()
  const min = () => Number.isFinite(props.min) ? props.min! : 0
  const max = () => Math.max(min(), Number.isFinite(props.max) ? props.max! : 100)
  const step = () => Number.isFinite(props.step) && props.step! > 0 ? props.step! : 1
  const [localValue, setLocalValue] = createSignal(props.defaultValue ?? min())
  let dragging = false
  const value = () => snapValue(props.value ?? localValue(), min(), max(), step())
  const fraction = () => max() > min() ? (value() - min()) / (max() - min()) : 0
  const readOnly = () => props.value !== undefined && !props.onValueChange
  const enabled = () => !props.disabled && !readOnly() && max() > min()
  const change = (next: number) => {
    if (!enabled()) return
    const snapped = snapValue(next, min(), max(), step())
    if (snapped === value()) return
    if (props.value === undefined) setLocalValue(snapped)
    props.onValueChange?.(snapped)
  }
  const pointer = (payload: NativeEventPayload<'pointerDown' | 'pointerMove' | 'pointerUp'>) => {
    const next = sliderPointerValue(payload.localX, payload.width, min(), max(), step())
    if (next !== undefined) change(next)
  }
  const onKey = (payload: unknown) => {
    const key = keyName(payload)
    if (key === 'ArrowRight' || key === 'ArrowUp') change(value() + step())
    else if (key === 'ArrowLeft' || key === 'ArrowDown') change(value() - step())
    else if (key === 'PageUp') change(value() + step() * 10)
    else if (key === 'PageDown') change(value() - step() * 10)
    else if (key === 'Home') change(min())
    else if (key === 'End') change(max())
  }
  return <focusScope id={props.id} role="slider" accessibleName={props.accessibleName}
    numericValue={value()} minimumValue={min()} maximumValue={max()}
    valueStep={step()} orientation="horizontal" canIncrement={enabled()} canDecrement={enabled()}
    canSetValue={enabled()} readOnly={readOnly()} enabled={enabled()} keyboardActivation="none"
    onKey={onKey} onSemanticAction={(payload) => {
      if (payload.action === 'increment') change(value() + step())
      else if (payload.action === 'decrement') change(value() - step())
      else if (payload.action === 'setValue' && typeof payload.value === 'number') change(payload.value)
    }}
    width={props.width ?? 240} height={props.height ?? 24}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}>
    <touchArea width="100%" height="100%" enabled={enabled()} mouseCursor={enabled() ? 'pointer' : 'notAllowed'}
      onPointerDown={(payload) => { dragging = enabled(); pointer(payload) }}
      onMoved={(payload) => { if (dragging) pointer(payload) }}
      onPointerUp={(payload) => { if (dragging) pointer(payload); dragging = false }}
      onPointerCancel={() => { dragging = false }}>
      <container width="100%" height="100%" position="relative">
        <rectangle width="100%" height={6} position="absolute" inset={{ top: 9, start: 0 }}
          background={theme().muted} radii={3} focusBorderColor={theme().focusRing}>
          <rectangle width={`${fraction() * 100}%`} height="100%" background={theme().primary} radii={3} />
        </rectangle>
        <row width="100%" height="100%" alignItems="center">
          <container grow={Math.max(0.00001, fraction())} />
          <rectangle width={16} height={16} radii={8} shrink={0} background={theme().background}
            border={{ width: 2, color: theme().primary }} opacity={enabled() ? 1 : 0.5} />
          <container grow={Math.max(0.00001, 1 - fraction())} />
        </row>
      </container>
    </touchArea>
  </focusScope>
}
