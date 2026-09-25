import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'
import type { SelectOption } from '../shared/select-options'
import { Select } from './select'

/** Props for the interactive Argui-rendered combo used by NativeSelect. */
export interface NativeSelectProps {
  /** Stable native key used for the combo and its popup. */
  id: string
  /** Accessible name for the selection control. */
  label: string
  /** Available values, optionally grouped or disabled. */
  options: readonly SelectOption[]
  /** Controlled selected value; omit it to use `defaultValue` and internal state. */
  value?: string
  /** Initial selected value used only when the component is uncontrolled. */
  defaultValue?: string
  /** Called with the selected option after user selection. */
  onChange?: (value: string) => void
  /** Palette used by the combo and popup. */
  theme: Palette
  /** Placeholder shown before a choice is made. */
  placeholder?: string
  /** Whether the control is disabled. */
  disabled?: boolean
  /** Control and popup width in pixels. */
  width?: number
}

/** Renders the existing accessible Argui combo behavior for native-select use cases. */
export function NativeSelect(props: NativeSelectProps): JSX.Element {
  return <Select id={props.id} label={props.label} options={props.options} value={props.value}
    defaultValue={props.defaultValue} onChange={props.onChange} theme={props.theme}
    placeholder={props.placeholder} disabled={props.disabled} width={props.width} />
}
