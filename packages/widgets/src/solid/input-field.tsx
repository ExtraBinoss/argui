import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { inputText } from '../shared/input-text'
import type { InputFieldProps } from '../shared/types'
import { useWidgetIcons } from './assets'

export { inputText } from '../shared/input-text'

/** Renders a themed native text input with an optional embedded search icon. */
export function InputField(props: InputFieldProps): JSX.Element {
  const [focused, setFocused] = createSignal(false)
  const icons = useWidgetIcons()
  return <column width="fill" gap={6}>
    {props.showLabel ? <text text={props.label} color={props.theme.muted} font_size={12} /> : null}
    <rectangle width="fill" height={42} background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
      border_color={props.invalid ? props.theme.destructive : focused() ? props.theme.accent : props.theme.border}
      border_width={focused() ? 2 : 1} radius={8}>
      <row width="fill" height="fill" padding_left={10} padding_right={10} gap={8} align_items="center">
        {props.search && icons.search ? <svg source={icons.search} color={props.theme.muted}
          width={17} height={17} /> : null}
        <textInput key={props.id} width="fill" height={22} value={props.value}
          placeholder={props.placeholder ?? ''} label={props.label} search={!!props.search}
          enabled={!props.disabled} read_only={!!props.readOnly} invalid={!!props.invalid} background="#00000000"
          text_color={props.disabled ? props.theme.muted : props.theme.foreground}
          placeholder_color={props.theme.muted} caret_color={props.theme.accent}
          selection_color={props.theme.accent}
          onFocus={() => setFocused(true)} onBlur={() => setFocused(false)} onInput={(payload) => {
            const value = inputText(payload)
            if (value !== undefined) props.onChange(value)
          }} onSubmit={(payload) => {
            const value = inputText(payload)
            props.onSubmit?.(value ?? props.value)
          }} />
      </row>
    </rectangle>
  </column>
}
