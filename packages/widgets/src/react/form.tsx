/** @jsxImportSource @argui/react */
import { createContext, createElement, useCallback, useContext, useEffect, useRef, useState,
  type ReactElement, type ReactNode } from 'react'
import { InputEditController } from '../shared/input-edit'
import {
  foundationIFieldId, foundationIValidateField, foundationIValidateForm,
  type FormErrors, type FormFieldRule, type FormValues, type FormValidator, type FormValidators,
} from '../shared/foundation-i'
import { selectionTint, type Palette } from '../shared/theme'

/** When field errors are calculated during interaction. */
export type FormValidationMode = 'submit' | 'change' | 'blur'

interface FormContextValue {
  id(): string
  theme(): Palette
  values(): FormValues
  errors(): FormErrors
  disabled(): boolean
  busy(): boolean
  submitted(): boolean
  registerField(name: string, rule: FormFieldRule): () => void
  setValue(name: string, value: string): void
  validateField(name: string): void
  submit(): void
}

interface FormFieldContextValue {
  name: string
  id(): string
  label(): string
  description(): string | undefined
  required(): boolean
  disabled(): boolean
  value(): string
  error(): string | undefined
  invalid(): boolean
  setValue(value: string): void
  blur(): void
  submit(): void
}

const FormContext = createContext<FormContextValue | null>(null)
const FormFieldContext = createContext<FormFieldContextValue | null>(null)

/** Props for a controlled or uncontrolled string-valued form. */
export interface FormProps {
  /** Stable prefix used to generate native input and accessibility relation keys. */
  id: string
  /** Palette shared by labels, controls, messages, and buttons. */
  theme: Palette
  /** Accessible name for the form's native group. */
  label: string
  /** Child fields and submit action. */
  children: ReactNode
  /** Current values; provide this with `onValuesChange` for controlled use. */
  values?: FormValues
  /** Initial values used only while `values` is omitted. */
  defaultValues?: FormValues
  /** Validators keyed by field name. */
  validators?: FormValidators
  /** When to show validation feedback. Defaults to submit. */
  validationMode?: FormValidationMode
  /** Disables every field and submit action. */
  disabled?: boolean
  /** Blocks duplicate submissions while an application request is pending. */
  busy?: boolean
  /** Called after any field value changes. */
  onValuesChange?: (values: FormValues) => void
  /** Called only when submission passes all registered validation rules. */
  onSubmit?: (values: FormValues) => void
}

/** Coordinates field values and synchronous validation without a form-library dependency. */
export function ReactForm(props: FormProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState<FormValues>(props.defaultValues ?? {})
  const values = () => props.values !== undefined ? props.values : uncontrolled
  const [errors, setErrors] = useState<FormErrors>({})
  const [submitted, setSubmitted] = useState(false)
  const fields = useRef(new Map<string, FormFieldRule>())
  const valuesRef = useRef<FormValues>(values())
  const validatorsRef = useRef<FormValidators>(props.validators ?? {})
  valuesRef.current = values()
  validatorsRef.current = props.validators ?? {}
  const validateAll = (current = valuesRef.current) => {
    const rules = Object.fromEntries(fields.current)
    const next = foundationIValidateForm(current, validatorsRef.current, rules)
    setErrors(next)
    return next
  }
  const validateField = (name: string) => {
    if (props.validationMode !== 'blur' && !submitted) return
    const message = foundationIValidateField(name, valuesRef.current, validatorsRef.current, fields.current.get(name))
    setErrors((current) => {
      const next = { ...current }
      if (message) next[name] = message
      else delete next[name]
      return next
    })
  }
  const setValue = (name: string, value: string) => {
    const next = { ...valuesRef.current, [name]: value }
    valuesRef.current = next
    if (props.values === undefined) setUncontrolled(next)
    props.onValuesChange?.(next)
    if (props.validationMode === 'change' || submitted) validateAll(next)
  }
  const submit = () => {
    if (props.disabled || props.busy) return
    setSubmitted(true)
    const nextErrors = validateAll()
    if (Object.keys(nextErrors).length === 0) props.onSubmit?.(valuesRef.current)
  }
  const registerField = useCallback((name: string, rule: FormFieldRule) => {
    fields.current.set(name, rule)
    return () => fields.current.delete(name)
  }, [])
  const context: FormContextValue = {
    id: () => props.id, theme: () => props.theme, values, errors: () => errors,
    disabled: () => !!props.disabled, busy: () => !!props.busy, submitted: () => submitted,
    registerField, setValue, validateField, submit,
  }
  return createElement(FormContext.Provider, { value: context },
    <focusScope width="fill" role="group" accessible_name={props.label} enabled={!props.disabled}>
      <column width="fill" gap={props.theme.controlPadding * 1.5}>{props.children}</column>
    </focusScope>) as ReactElement
}

/** Props for a named field that participates in the surrounding form state. */
export interface FormFieldProps {
  /** Stable field name used as the key in values, validators, and errors. */
  name: string
  /** Visible and accessible label used in default required feedback. */
  label?: string
  /** Visible description referenced by the text control. */
  description?: string
  /** Whether an empty or whitespace-only value is rejected. */
  required?: boolean
  /** Optional synchronous field-level validation rule. */
  validate?: FormValidator
  /** Label, control, description, and error components. */
  children: ReactNode
}

/** Registers a field rule and exposes its state to descendants. */
export function ReactFormField(props: FormFieldProps): ReactElement {
  const form = useContext(FormContext)
  if (!form) throw new Error('ReactFormField must be rendered inside ReactForm')
  useEffect(() => form.registerField(props.name, {
    label: props.label, required: props.required, validate: props.validate,
  }), [form.registerField, props.name, props.label, props.required, props.validate])
  const field: FormFieldContextValue = {
    name: props.name,
    id: () => foundationIFieldId(form.id(), props.name),
    label: () => props.label ?? props.name,
    description: () => props.description,
    required: () => !!props.required,
    disabled: () => form.disabled(),
    value: () => form.values()[props.name] ?? '',
    error: () => form.errors()[props.name],
    invalid: () => !!form.errors()[props.name],
    setValue: (value) => form.setValue(props.name, value),
    blur: () => form.validateField(props.name),
    submit: form.submit,
  }
  return createElement(FormFieldContext.Provider, { value: field }, props.children) as ReactElement
}

/** Returns the current field identifiers, value, validation state, and actions. */
export function useFormField(): FormFieldContextValue {
  const field = useContext(FormFieldContext)
  if (!field) throw new Error('useFormField must be called inside ReactFormField')
  return field
}

/** Props for the visual and semantic wrapper around one field. */
export interface FormItemProps {
  /** Field contents. */
  children: ReactNode
  /** Gap between label, control, description, and message. Defaults to six pixels. */
  gap?: number
}

/** Groups one field's label, editor, help text, and validation message. */
export function ReactFormItem(props: FormItemProps): ReactElement {
  const field = useFormField()
  return <focusScope width="fill" role="group" accessible_name={field.label()} enabled={!field.disabled()} focusable={false}>
    <column width="fill" gap={props.gap ?? 6}>{props.children}</column>
  </focusScope>
}

/** Props for the visible label associated with the current field. */
export interface FormLabelProps { /** Override the label stored on `FormField`. */ text?: string }

/** Renders a keyed visible label with invalid and required styling. */
export function ReactFormLabel(props: FormLabelProps = {}): ReactElement {
  const field = useFormField()
  const form = useContext(FormContext)!
  const label = () => props.text ?? field.label()
  return <row nativeKey={`${field.id()}-label`} role="text" accessible_name={label()} align_items="center" gap={4}
    opacity={field.disabled() ? 0.52 : 1}>
    <text text={label()} color={field.invalid() ? form.theme().destructive : form.theme().foreground}
      font_size={form.theme().controlFontSize} weight={500} accessible_hidden={true} />
    {field.required() ? <text text="*" color={form.theme().destructive} font_size={form.theme().controlFontSize}
      accessible_hidden={true} /> : null}
  </row>
}

/** Props for a structural wrapper around the native field control. */
export interface FormControlProps { /** Native control or custom field content. */ children: ReactNode }

/** Adds a full-width control slot; `FormInput` supplies the native label and error relations. */
export function ReactFormControl(props: FormControlProps): ReactElement {
  return <container width="fill" min_width={0}>{props.children}</container>
}

/** Props for the built-in single- or multiline native form editor. */
export interface FormInputProps {
  /** Placeholder shown while the field is empty. */
  placeholder?: string
  /** Renders a multiline editor when true. */
  multiline?: boolean
  /** Whether the value can be read but not edited. */
  readOnly?: boolean
  /** Disables this control in addition to the form's disabled state. */
  disabled?: boolean
  /** Editor height; multiline controls default to five text lines. */
  height?: number
  /** Whether Enter in a single-line editor submits the form. Defaults to true. */
  submitOnEnter?: boolean
}

/** Binds the native text editor to its field value, labels, and validation state. */
export function ReactFormInput(props: FormInputProps = {}): ReactElement {
  const field = useFormField()
  const form = useContext(FormContext)!
  const [focused, setFocused] = useState(false)
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(field.value())
  const disabled = field.disabled() || !!props.disabled
  const describedBy = [field.description() ? `${field.id()}-description` : undefined,
    field.error() ? `${field.id()}-message` : undefined].filter(Boolean).join(' ') || undefined
  const multiline = !!props.multiline
  const labelId = `${field.id()}-label`
  const height = props.height ?? (multiline ? Math.max(96, form.theme().controlFontSize * 6 + form.theme().controlPadding * 2) : form.theme().inputHeight)
  return <rectangle width="fill" height={height} clip={true}
    background={disabled ? form.theme().surfaceRaised : form.theme().surface}
    border_color={field.invalid() ? form.theme().destructive : focused ? form.theme().accent : form.theme().border}
    border_width={1} radius={form.theme().controlRadius}>
    <textInput nativeKey={field.id()} role={multiline ? 'text_area' : 'text_input'} width="fill" height="fill" clip={true}
      multiline={multiline} value={field.value()} placeholder={props.placeholder ?? ''}
      label={field.label()} description={field.description()} labelled_by={labelId} described_by={describedBy}
      enabled={!disabled} read_only={!!props.readOnly} required={field.required()} invalid={field.invalid()}
      background="#00000000" text_color={disabled ? form.theme().muted : form.theme().foreground}
      placeholder_color={form.theme().muted} caret_color={form.theme().accent} selection_color={selectionTint(form.theme().accent)}
      onFocus={() => setFocused(true)} onBlur={() => { setFocused(false); field.blur() }}
      onEdit={(payload) => edits.current!.apply(payload, field.value(), field.setValue)}
      onSubmit={multiline || props.submitOnEnter === false ? undefined : () => form.submit()} />
  </rectangle>
}

/** Props for a native submit action. */
export interface FormSubmitProps {
  /** Visible and accessible button label. */ label: string
  /** Optional stable key for the native button target. */ id?: string
  /** Visual treatment. Defaults to the accent primary action. */ variant?: 'primary' | 'secondary'
}

/** Validates the form and calls its submit callback only when every field passes. */
export function ReactFormSubmit(props: FormSubmitProps): ReactElement {
  const form = useContext(FormContext)
  if (!form) throw new Error('ReactFormSubmit must be rendered inside ReactForm')
  const [hovered, setHovered] = useState(false)
  const [pressed, setPressed] = useState(false)
  const [focused, setFocused] = useState(false)
  const disabled = form.disabled() || form.busy()
  const fill = props.variant === 'secondary'
    ? pressed ? form.theme().surfacePressed : hovered ? form.theme().surfaceHover : form.theme().surfaceRaised
    : pressed ? form.theme().accentPressed : hovered ? form.theme().accentHover : form.theme().accent
  const color = props.variant === 'secondary' ? form.theme().foreground : form.theme().accentText
  return <focusScope nativeKey={props.id} role="button" accessible_name={props.label} enabled={!disabled} busy={form.busy()}
    keyboard_activation="enter_or_space" onClick={() => form.submit()}
    onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}>
    <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => setHovered(true)} onPointerLeave={() => { setHovered(false); setPressed(false) }}
      onPointerDown={() => setPressed(true)} onPointerUp={() => setPressed(false)} onPointerCancel={() => setPressed(false)}>
      <rectangle height={form.theme().inputHeight} background={fill} border_color={focused ? form.theme().foreground : form.theme().border}
        border_width={props.variant === 'secondary' || focused ? 1 : 0} radius={form.theme().controlRadius} opacity={disabled ? 0.52 : 1}>
        <row width="fill" height="fill" padding_left={form.theme().controlPadding} padding_right={form.theme().controlPadding}
          align_items="center" justify_content="center">
          <text text={props.label} color={color} font_size={form.theme().controlFontSize} weight={600} />
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}

/** Props for visible field instructions. */
export interface FormDescriptionProps { /** Optional replacement for the text set on `FormField`. */ text?: string }

/** Renders muted instructions referenced by the associated native editor. */
export function ReactFormDescription(props: FormDescriptionProps = {}): ReactElement | null {
  const field = useFormField()
  const form = useContext(FormContext)!
  const text = props.text ?? field.description()
  return text ? <text nativeKey={`${field.id()}-description`} width="fill" text={text} color={form.theme().muted} font_size={12} /> : null
}

/** Props for the current validation message or an optional fallback hint. */
export interface FormMessageProps { /** Fallback shown until this field has a validation error. */ text?: string }

/** Announces the field's current validation error and omits an empty message. */
export function ReactFormMessage(props: FormMessageProps = {}): ReactElement | null {
  const field = useFormField()
  const form = useContext(FormContext)!
  const message = field.error() ?? props.text
  return message ? <text nativeKey={`${field.id()}-message`} width="fill" text={message}
    color={field.error() ? form.theme().destructive : form.theme().muted} font_size={12}
    role={field.error() ? 'alert' : 'text'} live={field.error() ? 'polite' : 'off'} /> : null
}
