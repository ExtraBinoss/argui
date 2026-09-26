/** @jsxImportSource @argui/react */
import { useId, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import { Button } from './button'
import { InputEditController, submittedText } from '../shared/input-edit'
import type { WidgetTheme } from '../shared/theme'
import type { InputFieldOptions } from '../shared/types'
import { useButtonGroup } from './button-group'

/** Props for the native React InputField. */
export type InputFieldProps = InputFieldOptions & { leading?: ReactNode; trailing?: ReactNode }

/** Renders a controlled or autonomous native text editor with a visible label. */
export function InputField(props: InputFieldProps): ReactElement {
  const generatedId = `argui-input-${useId().replaceAll(':', '')}`
  const id = props.id ?? generatedId
  const theme = useTheme<WidgetTheme>()
  const [uncontrolled, setUncontrolled] = useState(props.defaultValue ?? '')
  const value = props.value ?? uncontrolled
  const [revealed, setRevealed] = useState(false)
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(value)
  const readOnly = !!props.readOnly || (props.value !== undefined && !props.onValueChange)
  const inputType = props.type ?? 'text'
  const grouped = useButtonGroup() !== null

  return <column width={props.width ?? (grouped ? undefined : '100%')} height={props.height}
    minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin} gap={theme.spacing}>
    {props.label ? <text color={theme.textMuted} fontSize={12}>{props.label}</text> : null}
    <rectangle
      width="100%"
      height={grouped ? 38 : 40}
      padding={theme.spacing}
      background={grouped ? '#00000000' : theme.surface}
      border={{ width: 1, color: props.invalid ? theme.danger : grouped ? '#00000000' : theme.border }}
      radii={grouped ? 0 : theme.radius}
      focusBorderColor={grouped ? undefined : props.invalid ? theme.danger : theme.focusRing}
      opacity={props.disabled ? 0.55 : 1}
    >
      <row width="100%" height="100%" gap={theme.spacing} alignItems="center">
        {props.leading}
        <container grow={1} minWidth={0}>
          <textInput
            id={id}
            width="100%"
            height={20}
            value={value}
            placeholder={props.placeholder ?? ''}
            label={props.accessibleName ?? props.label}
            search={inputType === 'search'}
            privacy={inputType === 'password' ? revealed ? 'revealedPassword' : 'password' : 'public'}
            enabled={!props.disabled}
            readOnly={readOnly}
            invalid={!!props.invalid}
            background="#00000000"
            textColor={props.disabled ? theme.textMuted : theme.text}
            placeholderColor={theme.textMuted}
            caretColor={theme.primary}
            onEdit={(payload) => {
              edits.current!.apply(payload, value, (next) => {
                if (props.value === undefined) setUncontrolled(next)
                props.onValueChange?.(next)
              })
            }}
            onSubmit={(payload) => props.onSubmit?.(submittedText(payload) ?? value)}
          />
        </container>
        {props.trailing}
        {inputType === 'password' ? <Button
          id={`${id}-visibility`}
          variant="ghost"
          disabled={props.disabled}
          accessibleName={revealed ? 'Hide password' : 'Show password'}
          onClick={() => setRevealed((current) => !current)}
        >{revealed ? 'Hide' : 'Show'}</Button> : null}
      </row>
    </rectangle>
  </column>
}
