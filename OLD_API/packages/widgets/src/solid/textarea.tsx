import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { InputEditController } from '../shared/input-edit'
import { selectionTint } from '../shared/theme'
import type { Palette } from '../shared/theme'
import { Label } from './label'

/** Props for a themed multiline text editor with controlled and uncontrolled values. */
export interface TextareaProps {
  /** Stable native key used to retain the editor and connect its label. */
  id: string
  /** Palette used for the editor surface, border, text, and selection. */
  theme: Palette
  /** Controlled text value; omit it to use `defaultValue` and internal state. */
  value?: string
  /** Initial text used only when the editor is uncontrolled. */
  defaultValue?: string
  /** Called with the complete new text after each native edit. */
  onChange?: (value: string) => void
  /** Placeholder shown when the editor is empty. */
  placeholder?: string
  /** Accessible name, also used by an optionally visible label. */
  label?: string
  /** Description announced with the editor and rendered below it. */
  description?: string
  /** Label key to reference; defaults to `${id}-label` for a matching `Label`. */
  labelId?: string
  /** Whether to render the label above the editor. Defaults to false. */
  showLabel?: boolean
  /** Whether editing is disabled. */
  disabled?: boolean
  /** Whether the value can be read but not edited. */
  readOnly?: boolean
  /** Whether the field is required. */
  required?: boolean
  /** Whether the field is invalid. */
  invalid?: boolean
  /** Editor height in logical pixels. Defaults to four themed text lines. */
  height?: number
}

/** Renders an accessible multiline native text editor with a themed field surface. */
export function Textarea(props: TextareaProps): JSX.Element {
  const [uncontrolledValue, setUncontrolledValue] = createSignal(props.defaultValue ?? '')
  const value = () => props.value !== undefined ? props.value : uncontrolledValue()
  const edits = new InputEditController(value())
  const [focused, setFocused] = createSignal(false)
  const label = props.label ?? props.placeholder ?? 'Text area'
  const labelId = props.labelId ?? `${props.id}-label`
  const height = props.height ?? Math.max(80, props.theme.controlFontSize * 5 + props.theme.controlPadding * 2)
  return <column width="fill" gap={6}>
    {props.showLabel ? <Label id={labelId} htmlFor={props.id} text={label} theme={props.theme}
      disabled={props.disabled} required={props.required} /> : null}
    <rectangle width="fill" height={height}
      background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
      border_color={props.invalid ? props.theme.destructive : focused() ? props.theme.accent : props.theme.border}
      border_width={1} radius={props.theme.controlRadius}>
      <textInput key={props.id} role="text_area" width="fill" height="fill" clip={true} multiline={true}
        value={value()} placeholder={props.placeholder ?? ''} label={label} description={props.description}
        labelled_by={labelId} enabled={!props.disabled} read_only={!!props.readOnly}
        required={!!props.required} invalid={!!props.invalid} background="#00000000"
        text_color={props.disabled ? props.theme.muted : props.theme.foreground}
        placeholder_color={props.theme.muted} caret_color={props.theme.accent}
        selection_color={selectionTint(props.theme.accent)}
        onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
        onEdit={(payload) => edits.apply(payload, value(), (next) => {
          if (props.value === undefined) setUncontrolledValue(next)
          props.onChange?.(next)
        })} />
    </rectangle>
    {props.description ? <text text={props.description} color={props.theme.muted} font_size={12}
      accessible_hidden={true} /> : null}
  </column>
}
