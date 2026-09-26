/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import { Select as ReactSelect, type Palette } from '@argui/widgets/react'

export const choices = ['Vulkan', 'DirectX 12', 'Metal', 'WebGPU'] as const

/** Shows controlled native selection with the same options as Solid. */
export function ReactSelectPage(props: { theme: Palette; value: string; onChange: (value: string) => void }): ReactElement {
  return (
    <column gap={14}>
      <text width="fill" text="Choose the backend for your next render."
        color={props.theme.muted} font_size={14} />
      <ReactSelect id="topic-select" label="Choose a backend" options={choices} value={props.value}
        theme={props.theme} onChange={props.onChange} />
      <ReactSelect id="topic-grouped" label="Grouped release channel" theme={props.theme}
        placeholder="Choose a channel" defaultValue="stable"
        options={[{ value: 'stable', label: 'Stable', group: 'Recommended' },
          { value: 'preview', label: 'Preview', group: 'Recommended' },
          { value: 'nightly', label: 'Nightly', group: 'Experimental' },
          { value: 'retired', label: 'Retired', group: 'Experimental', disabled: true }]} />
      <text text={`Selected ${props.value}`} color={props.theme.muted} font_size={13} />
    </column>
  )
}
