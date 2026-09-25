import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Combobox } from '../../../../packages/widgets/src/solid/combobox'

const frameworks = [
  { value: 'next.js', label: 'Next.js', group: 'React' },
  { value: 'remix', label: 'Remix', group: 'React' },
  { value: 'sveltekit', label: 'SvelteKit', group: 'Svelte' },
  { value: 'astro', label: 'Astro', group: 'Other' },
  { value: 'nuxt.js', label: 'Nuxt.js', group: 'Vue' },
  { value: 'angular', label: 'Angular', group: 'Other', disabled: true },
]

/** Demonstrates searchable grouped options, controlled selection, keyboard movement, and clearing. */
export function ComboboxPage(props: { theme: Palette }): JSX.Element {
  const [framework, setFramework] = createSignal('next.js')
  return <column width="fill" gap={14}>
    <text width="fill" text="Type to filter grouped options. Use Up and Down to move, Enter to select, Escape to close, or the clear button to reset the value."
      color={props.theme.muted} font_size={13} />
    <Combobox id="foundation-j-framework" label="Framework" theme={props.theme} options={frameworks}
      value={framework()} onValueChange={setFramework} placeholder="Choose a framework…"
      searchPlaceholder="Search frameworks…" emptyLabel="No framework found." clearable />
    <text text={`Selected framework value: ${framework() || 'none'}`} color={props.theme.foreground} font_size={13} />
  </column>
}
