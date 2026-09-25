import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { InputEditController } from '../shared/input-edit'
import { inputText } from '../shared/input-text'
import { selectionTint } from '../shared/theme'
import type { InputFieldProps } from '../shared/types'
import { useWidgetIcons } from './assets'
import { Button } from './button'
import { useButtonGroupJoined } from './button-group'

export { inputText } from '../shared/input-text'

/** Renders a themed native text input with an optional embedded search icon. */
export function InputField(props: InputFieldProps): JSX.Element {
  const joined = useButtonGroupJoined()
  const [uncontrolled, setUncontrolled] = createSignal(props.defaultValue ?? '')
  const value = () => props.value ?? uncontrolled()
  const edits = new InputEditController(value())
  const [focused, setFocused] = createSignal(false)
  const [revealed, setRevealed] = createSignal(false)
  const icons = useWidgetIcons()
  return <column width="fill" gap={6}>
    {props.showLabel ? <text text={props.label} color={props.theme.muted} font_size={12} /> : null}
    <rectangle width="fill" height={joined ? 36 : props.theme.inputHeight} clip={true} background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
      border_color={props.invalid ? props.theme.destructive : focused() ? props.theme.accent : props.theme.border}
      border_width={joined && !focused() && !props.invalid ? 0 : 1} radius={joined ? 0 : props.theme.controlRadius}>
      <row width="fill" height="fill" min_width={0} padding_left={props.theme.controlPadding} padding_right={props.theme.controlPadding} gap={8} align_items="center">
        {props.search && icons.search ? <svg source={icons.search} color={props.theme.muted}
          width={17} height={17} /> : null}
        <container grow={1} min_width={0}>
          <textInput key={props.id} width="fill" height={22} clip={true} value={value()}
            placeholder={props.placeholder ?? ''} label={props.label} search={!!props.search}
            privacy={props.password ? revealed() ? 'revealed_password' : 'password' : 'public'}
            enabled={!props.disabled} read_only={!!props.readOnly} invalid={!!props.invalid} background="#00000000"
            text_color={props.disabled ? props.theme.muted : props.theme.foreground}
            placeholder_color={props.theme.muted} caret_color={props.theme.accent}
            selection_color={props.selectionColor ?? selectionTint(props.theme.accent)}
            onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
            onEdit={(payload) => { edits.apply(payload, value(), (next) => {
              if (props.value === undefined) setUncontrolled(next)
              props.onChange?.(next)
            }) }} onSubmit={(payload) => {
              const value = inputText(payload)
              props.onSubmit?.(value ?? (props.value ?? uncontrolled()))
            }} />
        </container>
        {props.password ? <Button id={`${props.id}-visibility`} label={revealed() ? 'Hide' : 'Show'}
          theme={props.theme} kind="ghost" disabled={props.disabled}
          onClick={() => setRevealed(!revealed())} /> : null}
      </row>
    </rectangle>
  </column>
}
