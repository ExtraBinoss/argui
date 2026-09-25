/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import {
  foundationKChartData,
  foundationKDimension,
  foundationKKey,
  type FoundationKChartProps,
} from '../shared/foundation-k'

export type ReactChartProps = FoundationKChartProps
export type ReactChartDatum = import('../shared/foundation-k').FoundationKChartDatum

/** Renders the same selectable, accessible native horizontal bar chart as Solid. */
export function ReactChart(props: ReactChartProps): ReactElement {
  const data = foundationKChartData(props.data)
  const [uncontrolled, setUncontrolled] = useState<string | null>(props.defaultSelectedId ?? null)
  const [activeIndex, setActiveIndex] = useState(() => Math.max(0, data.findIndex((datum) => datum.id === (props.selectedId ?? props.defaultSelectedId))))
  const selectedId = props.selectedId !== undefined ? props.selectedId : uncontrolled
  const chosen = data.find((datum) => datum.id === selectedId)
  const maxValue = Math.max(1, ...data.map((datum) => datum.value))
  const width = foundationKDimension(props.width, 420, 240)
  const height = foundationKDimension(props.height, 220, 100)
  const formatter = props.valueFormatter ?? ((value: number) => value.toLocaleString())
  const plotWidth = Math.max(32, width - 196)
  const currentIndex = Math.min(Math.max(0, activeIndex), Math.max(0, data.length - 1))
  const select = (index: number) => {
    const datum = data[index]
    if (!datum) return
    setActiveIndex(index)
    const next = selectedId === datum.id ? null : datum.id
    if (props.selectedId === undefined) setUncontrolled(next)
    props.onSelectionChange?.(next)
  }
  const onKey = (payload: unknown) => {
    const key = foundationKKey(payload)
    if (!key || data.length === 0) return
    let next = currentIndex
    if (key === 'ArrowDown' || key === 'ArrowRight') next = Math.min(data.length - 1, next + 1)
    else if (key === 'ArrowUp' || key === 'ArrowLeft') next = Math.max(0, next - 1)
    else if (key === 'Home') next = 0
    else if (key === 'End') next = data.length - 1
    else if (key === 'Enter' || key === ' ' || key === 'Space') { select(next); return }
    else return
    setActiveIndex(next)
  }
  return <column width={width} gap={10}>
    <column gap={3}>
      <text text={props.title} color={props.theme.foreground} font_size={16} weight={700} />
      {props.description ? <text text={props.description} color={props.theme.muted} font_size={12} /> : null}
    </column>
    <focusScope nativeKey={`${props.id}-list`} role="list_box" accessible_name={props.title}
      accessible_description={props.description} active_descendant={data[currentIndex] ? `${props.id}-option-${data[currentIndex]!.id}` : undefined}
      orientation="vertical" focusable={data.length > 0} enabled={data.length > 0} onKey={onKey}>
      <rectangle width="fill" height={height} background={props.theme.surface}
        border_color={props.theme.border} border_width={1} radius={props.theme.overlayRadius}>
        {data.length > 0
          ? <column width="fill" height="fill" gap={2} padding={10} justify_content="center">
            {data.map((datum, index) => {
              const selected = selectedId === datum.id
              const focused = currentIndex === index
              const barWidth = Math.max(2, plotWidth * datum.value / maxValue)
              return <focusScope key={`${props.id}-option-${datum.id}`} role="option"
                accessible_name={`${datum.label}, ${formatter(datum.value)}`} selected={selected}
                focusable={false} focus_on_tab_navigation={false} onClick={() => select(index)}>
                <touchArea mouse_cursor="pointer" onPointerEnter={() => setActiveIndex(index)}>
                  <row width="fill" height={32} gap={8} align_items="center" opacity={selected || !selectedId ? 1 : 0.72}>
                    <text width={82} text={datum.label} color={props.theme.foreground} font_size={12} />
                    <rectangle width={plotWidth} height={16} background={props.theme.surfaceRaised}
                      border_color={focused ? props.theme.accent : props.theme.border}
                      border_width={selected ? 2 : 1} radius={5}>
                      <rectangle width={barWidth} height={14} background={datum.color ?? props.theme.accent} radius={4} />
                    </rectangle>
                    <text width={88} text={formatter(datum.value)} color={selected ? props.theme.accent : props.theme.muted}
                      font_size={12} weight={selected ? 700 : 400} />
                  </row>
                </touchArea>
              </focusScope>
            })}
          </column>
          : <row width="fill" height="fill" align_items="center" justify_content="center">
            <text text="No chart data available." color={props.theme.muted} font_size={13} />
          </row>}
      </rectangle>
    </focusScope>
    <text text={chosen ? `Selected: ${chosen.label} — ${formatter(chosen.value)}` : 'Select a bar to inspect its value.'}
      color={props.theme.muted} font_size={12} role="status" live="polite" />
  </column>
}
