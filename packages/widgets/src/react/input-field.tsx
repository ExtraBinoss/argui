/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import { InputEditController } from '../shared/input-edit'
import { inputText } from '../shared/input-text'
import { selectionTint } from '../shared/theme'
import type { InputFieldProps } from '../shared/types'
import { useWidgetIcons } from './assets'
import { ReactButton } from './button'

/** Renders the same themed native input through the React adapter. */
export function ReactInputField(props: InputFieldProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState(props.defaultValue ?? '')
  const value = props.value ?? uncontrolled
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(value)
  const [focused, setFocused] = useState(false)
  const [revealed, setRevealed] = useState(false)
  const icons = useWidgetIcons()
  return <column width="fill" gap={6}>
    {props.showLabel ? <text text={props.label} color={props.theme.muted} font_size={12} /> : null}
    <rectangle width="fill" height={props.theme.inputHeight} clip={true} background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
      border_color={props.invalid ? props.theme.destructive : focused ? props.theme.accent : props.theme.border}
      border_width={1} radius={props.theme.controlRadius}>
      <row width="fill" height="fill" min_width={0} padding_left={props.theme.controlPadding} padding_right={props.theme.controlPadding} gap={8} align_items="center">
        {props.search && icons.search ? <svg source={icons.search} color={props.theme.muted}
          width={17} height={17} /> : null}
        <container grow={1} min_width={0}>
          <textInput nativeKey={props.id} width="fill" height={22} clip={true} value={value}
            placeholder={props.placeholder ?? ''} label={props.label} search={!!props.search}
            privacy={props.password ? revealed ? 'revealed_password' : 'password' : 'public'}
            enabled={!props.disabled} read_only={!!props.readOnly} invalid={!!props.invalid} background="#00000000"
            text_color={props.disabled ? props.theme.muted : props.theme.foreground}
            placeholder_color={props.theme.muted} caret_color={props.theme.accent}
            selection_color={props.selectionColor ?? selectionTint(props.theme.accent)}
            onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
            onEdit={(payload) => { edits.current!.apply(payload, value, (next) => {
              if (props.value === undefined) setUncontrolled(next)
              props.onChange?.(next)
            }) }} onSubmit={(payload) => {
              const submitted = inputText(payload)
              props.onSubmit?.(submitted ?? value)
            }} />
        </container>
        {props.password ? <ReactButton id={`${props.id}-visibility`} label={revealed ? 'Hide' : 'Show'}
          theme={props.theme} kind="ghost" disabled={props.disabled}
          onClick={() => setRevealed((current) => !current)} /> : null}
      </row>
    </rectangle>
  </column>
}
