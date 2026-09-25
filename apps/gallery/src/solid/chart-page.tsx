import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Chart } from '../../../../packages/widgets/src/solid/chart'

const visits = [
  { id: 'mon', label: 'Monday', value: 32 },
  { id: 'tue', label: 'Tuesday', value: 48 },
  { id: 'wed', label: 'Wednesday', value: 39 },
  { id: 'thu', label: 'Thursday', value: 64 },
  { id: 'fri', label: 'Friday', value: 55 },
]

/** Demonstrates visible bar values, pointer selection, and listbox keyboard navigation. */
export function ChartPage(props: { theme: Palette }): JSX.Element {
  const [selected, setSelected] = createSignal<string | null>('thu')
  return <column width="fill" gap={12}>
    <text text="Select a bar with the pointer or use the arrow keys; Home and End move to the first and last day. Values are printed beside each bar."
      color={props.theme.muted} font_size={13} />
    <Chart id="foundation-k-weekly-visits" title="Daily page views" description="Activity for the current work week"
      theme={props.theme} data={visits} selectedId={selected()} onSelectionChange={setSelected}
      valueFormatter={(value) => `${value} views`} />
    <text text={`Selected day: ${selected() ?? 'none'}`} color={props.theme.foreground} font_size={13} />
  </column>
}
