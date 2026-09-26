import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { RadioGroup } from '../../../../packages/widgets/src/solid/radio-group'

/** Shows a controlled vertical radio set, descriptions, and a disabled choice. */
export function RadioGroupPage(props: { theme: Palette }): JSX.Element {
  const [density, setDensity] = createSignal('comfortable')
  return <column width="fill" gap={15}>
    <text width="fill" text="Use arrow keys to move through enabled options; Home and End select the first and last choice."
      color={props.theme.muted} font_size={14} />
    <RadioGroup id="foundation-g-density" label="Display density" theme={props.theme}
      value={density()} onValueChange={setDensity} options={[
        { value: 'default', label: 'Default', description: 'Balanced spacing for everyday work.' },
        { value: 'comfortable', label: 'Comfortable', description: 'More room between rows.' },
        { value: 'compact', label: 'Compact', description: 'Tighter rows for dense lists.', disabled: true },
      ]} />
    <text text={`Current density: ${density()}`} color={props.theme.muted} font_size={12} />
  </column>
}
