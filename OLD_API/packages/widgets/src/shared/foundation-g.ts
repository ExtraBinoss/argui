import type { AssetRef } from '@argui/host'
import type { Palette } from './theme'

/** Checkbox state including the partially selected state. */
export type CheckboxValue = boolean | 'indeterminate'

/** Props shared by the Solid and React checkbox widgets. */
export interface CheckboxProps {
  /** Stable key for native reconciliation. */
  id: string
  /** Accessible name for the checkbox. */
  label: string
  /** Palette used for checkbox colors and corner radius. */
  theme: Palette
  /** Controlled checked value; omit it to use `defaultChecked`. */
  checked?: CheckboxValue
  /** Initial checked value used only when the checkbox is uncontrolled. */
  defaultChecked?: CheckboxValue
  /** Called after user input with the complete next checked state. */
  onCheckedChange?: (checked: CheckboxValue) => void
  /** Optional description announced with the checkbox. */
  description?: string
  /** Whether the checkbox is disabled. */
  disabled?: boolean
  /** Whether the checkbox is required. */
  required?: boolean
  /** Whether the checkbox value is invalid. */
  invalid?: boolean
}

/** One labeled choice inside a radio group. */
export interface RadioOption {
  /** Value emitted when the option is selected. */
  value: string
  /** Accessible and visible option label. */
  label: string
  /** Optional supporting text displayed beside the option. */
  description?: string
  /** Whether this option cannot be selected. */
  disabled?: boolean
}

/** Props shared by the Solid and React radio-group widgets. */
export interface RadioGroupProps {
  /** Stable key prefix for the group and its radio options. */
  id: string
  /** Accessible name for the radio group. */
  label: string
  /** Available values and their display labels. */
  options: readonly RadioOption[]
  /** Palette used for option circles, focus, and text. */
  theme: Palette
  /** Controlled option value; omit it to use `defaultValue`. */
  value?: string
  /** Initial option value used only when the group is uncontrolled. */
  defaultValue?: string
  /** Called after a different option is selected. */
  onValueChange?: (value: string) => void
  /** Whether every option is disabled. */
  disabled?: boolean
  /** Whether the selected value is invalid. */
  invalid?: boolean
  /** Layout direction and arrow-key axis for the radio options. */
  orientation?: 'horizontal' | 'vertical'
}

/** Size and border treatments supported by a toggle. */
export type ToggleSize = 'sm' | 'default' | 'lg'
export type ToggleVariant = 'default' | 'outline'

/** Props shared by the Solid and React toggle widgets. */
export interface ToggleProps {
  /** Stable key for native reconciliation. */
  id: string
  /** Accessible name for the toggle button. */
  label: string
  /** Palette used for toggle fills, borders, text, and focus. */
  theme: Palette
  /** Visible text; defaults to the accessible label. */
  text?: string
  /** Optional application-owned icon displayed before the text. */
  icon?: AssetRef
  /** Controlled pressed value; omit it to use `defaultPressed`. */
  pressed?: boolean
  /** Initial pressed value used only when the toggle is uncontrolled. */
  defaultPressed?: boolean
  /** Called after the pressed state changes. */
  onPressedChange?: (pressed: boolean) => void
  /** Whether the toggle is disabled. */
  disabled?: boolean
  /** Whether the toggle value is invalid. */
  invalid?: boolean
  /** Border treatment. Defaults to `default`. */
  variant?: ToggleVariant
  /** Control size. Defaults to `default`. */
  size?: ToggleSize
}

/** Returns a key name from a native key event and ignores key release events. */
export function foundationGKey(payload: unknown): string | undefined {
  if (typeof payload === 'object' && payload !== null && 'key' in payload) {
    if ('state' in payload && payload.state !== 'pressed') return undefined
    return typeof payload.key === 'string' ? payload.key : undefined
  }
  return typeof payload === 'string' ? payload : undefined
}

/** Returns a valid enabled default radio value, or no selection. */
export function foundationGInitialRadioValue(options: readonly RadioOption[], preferred?: string): string | undefined {
  if (preferred !== undefined && options.some((option) => option.value === preferred && !option.disabled)) {
    return preferred
  }
  return undefined
}

/** Returns an enabled option value in the requested wrapped direction. */
export function foundationGNextRadioValue(
  options: readonly RadioOption[],
  current: string | undefined,
  direction: -1 | 1,
): string | undefined {
  const enabled = options.filter((option) => !option.disabled)
  if (enabled.length === 0) return undefined
  const currentIndex = enabled.findIndex((option) => option.value === current)
  const nextIndex = (currentIndex + direction + enabled.length) % enabled.length
  return enabled[nextIndex]?.value
}
