/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import type { SemanticActionPayload } from '@argui/host'
import type { SliderProps } from '../shared/foundation-e'
import {
  adjustSliderThumb, normalizeSliderValues, sliderActionValue, sliderDomain,
  sliderKeyboardValue, sliderPointerRatio,
} from '../shared/foundation-e'

const thumbSize = 16
const thumbInset = thumbSize / 2
const trackThickness = 4

/** Provides a controlled or default-valued native slider with range support. */
export function Slider(props: SliderProps): ReactElement {
  const domain = sliderDomain(props.min, props.max, props.step)
  const orientation = props.orientation ?? 'horizontal'
  const width = dimension(props.width, orientation === 'horizontal' ? 280 : 40)
  const height = dimension(props.height, orientation === 'vertical' ? 180 : 40)
  const length = orientation === 'horizontal' ? width : height
  const trackLength = Math.max(0, length - thumbSize)
  const trackOrigin = thumbInset
  const [uncontrolled, setUncontrolled] = useState(normalizeSliderValues(
    props.defaultValue, domain.min, domain.max, domain.step,
  ))
  const [focusedIndex, setFocusedIndex] = useState(-1)
  const activeIndex = useRef(0)
  const dragging = useRef(false)

  const values = normalizeSliderValues(props.value ?? uncontrolled,
    domain.min, domain.max, domain.step)
  const fraction = (value: number) => (value - domain.min) / (domain.max - domain.min)
  const commit = (next: number[]) => {
    if (next.length === values.length && next.every((value, index) => value === values[index])) return
    if (props.value === undefined) setUncontrolled(next)
    props.onValueChange?.(next)
  }
  const setThumb = (index: number, next: number) => commit(adjustSliderThumb(
    values, index, next, domain.min, domain.max, domain.step,
  ))
  const pointerValue = (payload: unknown) => {
    const ratio = sliderPointerRatio(payload, orientation, thumbInset)
    return ratio === undefined ? undefined : domain.min + ratio * (domain.max - domain.min)
  }
  const beginDrag = (payload: unknown) => {
    if (props.disabled) return
    const target = pointerValue(payload)
    if (target === undefined) return
    activeIndex.current = values.reduce((nearest, value, index) =>
      Math.abs(value - target) < Math.abs(values[nearest]! - target) ? index : nearest, 0)
    dragging.current = true
    setThumb(activeIndex.current, target)
  }
  const moveDrag = (payload: unknown) => {
    const target = pointerValue(payload)
    if (!props.disabled && dragging.current && target !== undefined) setThumb(activeIndex.current, target)
  }
  const semanticAction = (index: number, payload: SemanticActionPayload) => {
    const next = sliderActionValue(payload, values[index]!, domain.min, domain.max, domain.step)
    if (next !== undefined) setThumb(index, next)
  }
  const handleKey = (index: number, payload: unknown) => {
    const next = sliderKeyboardValue(payload, values[index]!, domain.min, domain.max,
      domain.step, orientation)
    if (next !== undefined) setThumb(index, next)
  }

  return <touchArea width={width} height={height} enabled={!props.disabled}
    mouse_cursor={props.disabled ? 'not_allowed' : 'grab'} role="group"
    accessible_name={props.label} accessible_disabled={props.disabled}
    onPointerDown={beginDrag} onMoved={moveDrag}
    onPointerUp={() => { dragging.current = false }} onPointerCancel={() => { dragging.current = false }}>
    <container width={width} height={height}>
      {orientation === 'horizontal'
        ? <rectangle x={trackOrigin} y={(height - trackThickness) / 2}
          width={trackLength} height={trackThickness} radius={trackThickness / 2}
          background={props.theme.surfaceRaised} />
        : <rectangle x={(width - trackThickness) / 2} y={trackOrigin}
          width={trackThickness} height={trackLength} radius={trackThickness / 2}
          background={props.theme.surfaceRaised} />}
      {(() => {
        const start = values.length > 1 ? fraction(values[0]!) : 0
        const end = fraction(values[values.length - 1]!)
        const activeLength = Math.max(0, end - start) * trackLength
        if (activeLength <= 0) return null
        return orientation === 'horizontal'
          ? <rectangle x={trackOrigin + start * trackLength}
            y={(height - trackThickness) / 2} width={activeLength} height={trackThickness}
            radius={trackThickness / 2} background={props.theme.accent} />
          : <rectangle x={(width - trackThickness) / 2}
            y={trackOrigin + (1 - end) * trackLength} width={trackThickness} height={activeLength}
            radius={trackThickness / 2} background={props.theme.accent} />
      })()}
      {values.map((value, index) => {
        const position = trackOrigin + (orientation === 'horizontal'
          ? fraction(value) : 1 - fraction(value)) * trackLength - thumbInset
        const x = orientation === 'horizontal' ? position : (width - thumbSize) / 2
        const y = orientation === 'horizontal' ? (height - thumbSize) / 2 : position
        const name = values.length === 1 ? props.label : values.length === 2
          ? `${props.label} ${index === 0 ? 'minimum' : 'maximum'}`
          : `${props.label} thumb ${index + 1}`
        return <focusScope key={`${props.id}-thumb-${index}`} nativeKey={`${props.id}-thumb-${index}`}
          x={x} y={y} width={thumbSize} height={thumbSize} role="slider" accessible_name={name}
          orientation={orientation} numeric_value={value} minimum_value={domain.min}
          maximum_value={domain.max} value_step={domain.step} focusable={!props.disabled}
          enabled={!props.disabled} accessible_disabled={props.disabled}
          can_increment={!props.disabled} can_decrement={!props.disabled} can_set_value={!props.disabled}
          onFocus={() => setFocusedIndex(index)} onBlur={() => setFocusedIndex(-1)}
          onKey={(payload) => handleKey(index, payload)}
          onSemanticAction={(payload) => semanticAction(index, payload)}>
          <rectangle width="fill" height="fill" radius={thumbInset}
            background={props.theme.surface} border_color={focusedIndex === index
              ? props.theme.accent : props.theme.border} border_width={focusedIndex === index ? 2 : 1}
            shadow_blur={2} shadow_color={props.theme.overlayShadow} />
        </focusScope>
      })}
    </container>
  </touchArea>
}

function dimension(value: number | undefined, fallback: number): number {
  const requested = value ?? 0
  return Number.isFinite(requested) && requested > 0 ? requested : fallback
}
