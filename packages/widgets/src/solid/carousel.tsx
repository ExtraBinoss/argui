import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { surfaceECarouselIndex, surfaceEDragPixels, surfaceEKey, type SurfaceECarouselProps } from '../shared/surface-e'
import { Button } from './button'

export type CarouselProps = SurfaceECarouselProps<JSX.Element>

/** Displays one slide at a time with buttons, arrow keys, and touch swipes. */
export function Carousel(props: CarouselProps): JSX.Element {
  const [localIndex, setLocalIndex] = createSignal(normalizeIndex(props.defaultIndex ?? 0, props.slides.length))
  const [focused, setFocused] = createSignal(false)
  const [dragDistance, setDragDistance] = createSignal(0)
  const [swiped, setSwiped] = createSignal(false)
  const orientation = props.orientation ?? 'horizontal'
  const height = dimension(props.height, 220)
  const count = () => props.slides.length
  const index = () => normalizeIndex(props.index ?? localIndex(), count())
  const commit = (nextIndex: number) => {
    const normalized = normalizeIndex(nextIndex, count())
    if (normalized === index()) return
    if (props.index === undefined) setLocalIndex(normalized)
    props.onValueChange?.(normalized)
  }
  const move = (direction: -1 | 1) => commit(surfaceECarouselIndex(index(), direction, count(), !!props.loop))
  const handleKey = (payload: unknown) => {
    const key = surfaceEKey(payload)
    if (key === (orientation === 'horizontal' ? 'ArrowLeft' : 'ArrowUp')) move(-1)
    else if (key === (orientation === 'horizontal' ? 'ArrowRight' : 'ArrowDown')) move(1)
    else if (key === 'Home') commit(0)
    else if (key === 'End') commit(count() - 1)
  }
  const drag = (payload: unknown) => {
    if (swiped()) return
    const nextDistance = dragDistance() + surfaceEDragPixels(payload)
    setDragDistance(nextDistance)
    const forward = nextDistance < -48
    const backward = nextDistance > 48
    if (forward || backward) {
      setSwiped(true)
      move(forward ? 1 : -1)
    }
  }
  const resetDrag = () => { setDragDistance(0); setSwiped(false) }
  const panel = props.slides[index()]
  const label = props.label ?? 'Carousel'
  return <column width="fill" gap={10}>
    <focusScope key={`${props.id}-carousel`} role="group" accessible_name={label}
      accessible_description="Use the arrow keys to change slides." focusable={true}
      onFocus={() => setFocused(true)} onBlur={() => setFocused(false)} onKey={handleKey}>
      <touchArea key={`${props.id}-viewport`} width="fill" height={height}
        mouse_cursor="grab" onPointerDown={resetDrag} onPointerUp={resetDrag} onPointerCancel={resetDrag}
        onDragX={orientation === 'horizontal' ? drag : undefined}
        onDragY={orientation === 'vertical' ? drag : undefined}>
        <rectangle width="fill" height="fill" background={props.theme.surface}
          border_color={focused() ? props.theme.accent : props.theme.border} border_width={1}
          radius={props.theme.overlayRadius}>
          <column width="fill" height="fill" padding={18} align_items="center" justify_content="center"
            role="group" accessible_name={count() ? `Slide ${index() + 1} of ${count()}` : 'No slides'}
            position_in_set={count() ? index() + 1 : undefined} set_size={count() || undefined}>
            {panel ?? <text text="No slides" color={props.theme.muted} font_size={14} />}
          </column>
        </rectangle>
      </touchArea>
    </focusScope>
    <row width="fill" gap={8} align_items="center" justify_content="space_between">
      <Button id={`${props.id}-previous`} label="Previous slide" theme={props.theme}
        kind="outline" disabled={!props.loop && index() <= 0} onClick={() => move(-1)} />
      <text text={count() ? `Slide ${index() + 1} of ${count()}` : 'No slides'}
        color={props.theme.muted} font_size={12} />
      <Button id={`${props.id}-next`} label="Next slide" theme={props.theme}
        kind="outline" disabled={!props.loop && index() >= count() - 1} onClick={() => move(1)} />
    </row>
    <row width="fill" gap={5} align_items="center" justify_content="center">
      {props.slides.map((_, slideIndex) => <Button id={`${props.id}-indicator-${slideIndex}`} label={`Show slide ${slideIndex + 1}`}
        theme={props.theme} kind="ghost" size="sm" selected={index() === slideIndex}
        onClick={() => commit(slideIndex)} />)}
    </row>
  </column>
}

function normalizeIndex(value: number, count: number): number {
  if (count <= 0) return 0
  return Number.isFinite(value) ? Math.max(0, Math.min(count - 1, Math.trunc(value))) : 0
}

function dimension(value: number | undefined, fallback: number): number {
  return Number.isFinite(value) && (value ?? 0) > 0 ? value! : fallback
}
