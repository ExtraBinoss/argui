/** String-valued field data managed by the lightweight Argui form component. */
export type FormValues = Readonly<Record<string, string>>

/** One synchronous field validator; return a message to reject a value. */
export type FormValidator = (value: string, values: FormValues) => string | undefined

/** Validators indexed by field name. */
export type FormValidators = Readonly<Record<string, FormValidator>>

/** Validation messages indexed by field name. */
export type FormErrors = Readonly<Record<string, string>>

/** Required and custom validation rules registered by a mounted form field. */
export interface FormFieldRule {
  /** Visible label used in the default required message. */
  label?: string
  /** Whether an empty or whitespace-only value should fail validation. */
  required?: boolean
  /** Additional field-specific validation. */
  validate?: FormValidator
}

/** One toggle-group choice used by keyboard navigation. */
export interface ToggleGroupChoice {
  /** Stable value reported to the application. */
  value: string
  /** Whether keyboard navigation and activation should skip this choice. */
  disabled?: boolean
}

/** Returns a pressed native key name and ignores key-release events. */
export function foundationIKey(payload: unknown): string | undefined {
  if (typeof payload === 'object' && payload !== null && 'key' in payload) {
    const event = payload as { key?: unknown; state?: unknown }
    if (event.state !== undefined && event.state !== 'pressed') return undefined
    return typeof event.key === 'string' ? event.key : undefined
  }
  return typeof payload === 'string' ? payload : undefined
}

/** Finds the next enabled choice in a wrapped direction. */
export function foundationINextToggleValue(
  choices: readonly ToggleGroupChoice[],
  current: string | undefined,
  direction: -1 | 1,
): string | undefined {
  const enabled = choices.filter((choice) => !choice.disabled)
  if (enabled.length === 0) return undefined
  const index = enabled.findIndex((choice) => choice.value === current)
  return enabled[(index + direction + enabled.length) % enabled.length]?.value
}

/** Creates a stable native key from a form and field name. */
export function foundationIFieldId(formId: string, name: string): string {
  const suffix = name.replace(/[^a-zA-Z0-9_-]+/g, '-') || 'field'
  return `${formId}-${suffix}`
}

/** Validates one field using its local rule before the form-wide fallback validator. */
export function foundationIValidateField(
  name: string,
  values: FormValues,
  validators: FormValidators,
  rule?: FormFieldRule,
): string | undefined {
  const value = values[name] ?? ''
  if (rule?.required && !value.trim()) return `${rule.label || name} is required.`
  return rule?.validate?.(value, values) ?? validators[name]?.(value, values)
}

/** Validates every provided field and returns only failing messages. */
export function foundationIValidateForm(
  values: FormValues,
  validators: FormValidators,
  rules: Readonly<Record<string, FormFieldRule>> = {},
): Record<string, string> {
  const errors: Record<string, string> = {}
  const names = new Set([...Object.keys(validators), ...Object.keys(rules)])
  for (const name of names) {
    const message = foundationIValidateField(name, values, validators, rules[name])
    if (message) errors[name] = message
  }
  return errors
}
