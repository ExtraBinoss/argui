/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactCombobox as Combobox } from '../../../../packages/widgets/src/react/combobox'

const frameworks = [
  { value: 'next.js', label: 'Next.js' },
  { value: 'sveltekit', label: 'SvelteKit' },
  { value: 'nuxt.js', label: 'Nuxt.js' },
  { value: 'remix', label: 'Remix' },
  { value: 'astro', label: 'Astro' },
]

/** Demonstrates a searchable framework picker with an optional selected value. */
export function ComboboxPage(props: { theme: Palette }): ReactElement {
  const [framework, setFramework] = useState('')
  return <column width="fill" gap={14}>
    <Combobox id="foundation-j-framework" label="Framework" theme={props.theme} options={frameworks}
      value={framework} onValueChange={setFramework} placeholder="Select a framework"
      emptyLabel="No framework found." width={300} showLabel={false} />
  </column>
}
