import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { NativeSelect, type Palette } from '@argui/widgets/solid'

/** Demonstrates selection while stating the missing platform-native menu behavior. */
export function NativeSelectPage(props: { theme: Palette }): JSX.Element {
  const options = [
    { value: 'Todo', group: 'Open' },
    { value: 'In progress', group: 'Open' },
    { value: 'Done', group: 'Closed' },
    { value: 'Cancelled', group: 'Closed', disabled: true },
  ]
  const [status, setStatus] = createSignal('Todo')
  return <column width="fill" gap={14}>
    <text width="fill" text="Selection supports grouped and disabled options. Argui renders its native combo surface rather than an OS or browser select menu."
      color={props.theme.muted} font_size={14} />
    <NativeSelect id="foundation-f-status" label="Project status" options={options}
      theme={props.theme} value={status()} onChange={setStatus} />
    <text text={`Selected status: ${status()}`} color={props.theme.foreground} font_size={13} />
  </column>
}
