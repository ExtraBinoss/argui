import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { SemanticActionPayload } from '@argui/host'
import { surfaceEClampSplit, surfaceEDragPixels, surfaceEKey, surfaceEResizeByPixels, type SurfaceEResizableProps } from '../shared/surface-e'

/** A two-panel split layout with an accessible keyboard and pointer divider. */
export function Resizable(props: SurfaceEResizableProps<JSX.Element>): JSX.Element {
  const orientation = props.orientation ?? 'horizontal'
  const extent = dimension(orientation === 'horizontal' ? props.width : props.height,
    orientation === 'horizontal' ? 640 : 260)
  const crossExtent = dimension(orientation === 'horizontal' ? props.height : props.width,
    orientation === 'horizontal' ? 260 : 640)
  const handleSize = 14
  const min = Math.min(99.5, bound(props.minValue, 15))
  const max = Math.max(min + 0.5, Math.min(100, bound(props.maxValue, 85)))
  const step = Number.isFinite(props.step) && (props.step ?? 0) > 0 ? props.step! : 1
  const [localValue, setLocalValue] = createSignal(surfaceEClampSplit(props.defaultValue ?? 50, min, max))
  const [focused, setFocused] = createSignal(false)
  const value = () => surfaceEClampSplit(props.value ?? localValue(), min, max)
  const commit = (nextValue: number) => {
    const next = surfaceEClampSplit(nextValue, min, max)
    if (next === value()) return
    if (props.value === undefined) setLocalValue(next)
    props.onValueChange?.(next)
  }
  const adjust = (delta: number) => commit(value() + delta)
  const handleKey = (payload: unknown) => {
    const key = surfaceEKey(payload)
    if (key === 'Home') commit(min)
    else if (key === 'End') commit(max)
    else if (key === (orientation === 'horizontal' ? 'ArrowLeft' : 'ArrowUp')) adjust(-step)
    else if (key === (orientation === 'horizontal' ? 'ArrowRight' : 'ArrowDown')) adjust(step)
    else if (key === 'PageDown') adjust(-step * 10)
    else if (key === 'PageUp') adjust(step * 10)
  }
  const semanticAction = (payload: SemanticActionPayload) => {
    if (payload.action === 'increment') adjust(step)
    else if (payload.action === 'decrement') adjust(-step)
    else if (payload.action === 'set_value' && typeof payload.value === 'number') commit(payload.value)
  }
  const panelExtent = Math.max(0, extent - handleSize)
  const firstExtent = panelExtent * value() / 100
  const secondExtent = panelExtent - firstExtent
  const divider = <focusScope key={`${props.id}-separator`} role="separator"
    accessible_name={props.handleLabel ?? 'Resize panels'} orientation={orientation === 'horizontal' ? 'vertical' : 'horizontal'}
    numeric_value={value()} minimum_value={min} maximum_value={max} value_step={step}
    focusable={true} can_increment={true} can_decrement={true} can_set_value={true}
    onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
    onKey={handleKey} onSemanticAction={semanticAction}>
    {orientation === 'horizontal'
      ? <touchArea width={handleSize} height={crossExtent} mouse_cursor="col_resize"
        onDragX={(payload) => commit(surfaceEResizeByPixels(value(), surfaceEDragPixels(payload), panelExtent, min, max))}>
        <rectangle width="fill" height="fill" background="#00000000">
          <rectangle width={4} height={48} radius={2} background={focused() ? props.theme.accent : props.theme.border} />
        </rectangle>
      </touchArea>
      : <touchArea width={crossExtent} height={handleSize} mouse_cursor="row_resize"
        onDragY={(payload) => commit(surfaceEResizeByPixels(value(), surfaceEDragPixels(payload), panelExtent, min, max))}>
        <rectangle width="fill" height="fill" background="#00000000">
          <rectangle width={48} height={4} radius={2} background={focused() ? props.theme.accent : props.theme.border} />
        </rectangle>
      </touchArea>}
  </focusScope>
  return orientation === 'horizontal'
    ? <row width={extent} height={crossExtent} gap={0} role="group" accessible_name={props.label ?? 'Resizable panels'}>
      <rectangle width={firstExtent} height="fill" background={props.theme.surface} border_color={props.theme.border}
        border_width={1} radius={props.theme.overlayRadius}>
        <column width="fill" height="fill" padding={12} align_items="center" justify_content="center">{props.first}</column>
      </rectangle>
      {divider}
      <rectangle width={secondExtent} height="fill" background={props.theme.surface} border_color={props.theme.border}
        border_width={1} radius={props.theme.overlayRadius}>
        <column width="fill" height="fill" padding={12} align_items="center" justify_content="center">{props.second}</column>
      </rectangle>
    </row>
    : <column width={crossExtent} height={extent} gap={0} role="group" accessible_name={props.label ?? 'Resizable panels'}>
      <rectangle width="fill" height={firstExtent} background={props.theme.surface} border_color={props.theme.border}
        border_width={1} radius={props.theme.overlayRadius}>
        <column width="fill" height="fill" padding={12} align_items="center" justify_content="center">{props.first}</column>
      </rectangle>
      {divider}
      <rectangle width="fill" height={secondExtent} background={props.theme.surface} border_color={props.theme.border}
        border_width={1} radius={props.theme.overlayRadius}>
        <column width="fill" height="fill" padding={12} align_items="center" justify_content="center">{props.second}</column>
      </rectangle>
    </column>
}

function dimension(value: number | undefined, fallback: number): number {
  return Number.isFinite(value) && (value ?? 0) > 0 ? value! : fallback
}

function bound(value: number | undefined, fallback: number): number {
  return Number.isFinite(value) ? Math.max(0, Math.min(100, value!)) : fallback
}
