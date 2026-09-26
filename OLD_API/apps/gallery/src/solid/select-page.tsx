import type { JSX } from '@argui/solid/jsx-runtime'
import { Select, type Palette } from '@argui/widgets/solid'

export const choices = ['Vulkan', 'DirectX 12', 'Metal', 'WebGPU'] as const

/** Shows a controlled selector backed by the native popup and focus boundary. */
export function SelectPage(props: { theme: Palette; value: string; onChange: (value: string) => void }): JSX.Element {
  return (
    <column gap={14}>
      <text text="Choose the backend for your next render." width="fill" color={props.theme.muted} font_size={14} />
      <Select id="topic-select" label="Choose a backend" options={choices} value={props.value} theme={props.theme} onChange={props.onChange} />
      <Select id="topic-grouped" label="Grouped release channel" theme={props.theme}
        placeholder="Choose a channel" defaultValue="stable"
        options={[{ value: 'stable', label: 'Stable', group: 'Recommended' },
          { value: 'preview', label: 'Preview', group: 'Recommended' },
          { value: 'nightly', label: 'Nightly', group: 'Experimental' },
          { value: 'retired', label: 'Retired', group: 'Experimental', disabled: true }]} />
      <text text={`Selected ${props.value}`} color={props.theme.muted} font_size={13} />
    </column>
  )
}
