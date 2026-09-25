import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Checkbox } from '../../../../packages/widgets/src/solid/checkbox'

/** Shows controlled, indeterminate, required, and disabled checkbox states. */
export function CheckboxPage(props: { theme: Palette }): JSX.Element {
  const [accepted, setAccepted] = createSignal(false)
  const [allSelected, setAllSelected] = createSignal<'indeterminate' | boolean>('indeterminate')
  return <column width="fill" gap={15}>
    <text width="fill" text="Checkbox state is exposed to assistive technology and can be controlled, defaulted, or indeterminate."
      color={props.theme.muted} font_size={14} />
    <row gap={10} align_items="center">
      <Checkbox id="foundation-g-terms" label="Accept terms and conditions" theme={props.theme}
        checked={accepted()} required onCheckedChange={(value) => setAccepted(value === true)} />
      <text text="Accept terms and conditions" color={props.theme.foreground} font_size={14} />
    </row>
    <row gap={10} align_items="center">
      <Checkbox id="foundation-g-select-all" label="Select all files" theme={props.theme}
        checked={allSelected()} onCheckedChange={setAllSelected} />
      <column gap={3}>
        <text text="Select all files" color={props.theme.foreground} font_size={14} />
        <text text="Mixed state becomes checked on activation." color={props.theme.muted} font_size={12} />
      </column>
    </row>
    <row gap={10} align_items="center">
      <Checkbox id="foundation-g-disabled" label="Enable notifications" theme={props.theme}
        defaultChecked disabled />
      <text text="Enable notifications" color={props.theme.muted} font_size={14} />
    </row>
  </column>
}
