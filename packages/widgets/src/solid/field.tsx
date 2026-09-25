import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'
import { uniqueFieldErrorMessages } from '../shared/foundation-h'

/** Layout direction accepted by a field wrapper. */
export type FieldOrientation = 'vertical' | 'horizontal' | 'responsive'

/** Props for a semantically grouped form field set. */
export interface FieldSetProps {
  /** Colors and sizing for the field set. */
  theme: Palette
  /** Optional accessible name, usually the visible legend text. */
  legend?: string
  /** Fields, descriptions, and legends in this set. */
  children: JSX.Element
  /** Spacing between direct children. Defaults to a relaxed themed gap. */
  gap?: number
}

/** Groups related controls and exposes their optional legend to assistive technology. */
export function FieldSet(props: FieldSetProps): JSX.Element {
  return <focusScope width="fill" role="group" accessible_name={props.legend}>
    <column width="fill" gap={props.gap ?? props.theme.controlPadding * 1.5}>{props.children}</column>
  </focusScope>
}

/** Props for a visual field-set legend. */
export interface FieldLegendProps {
  /** Palette used for legend text. */
  theme: Palette
  /** Heading shown at the start of the field set. */
  text: string
  /** Smaller label styling for a legend used as an inline group title. */
  variant?: 'legend' | 'label'
}

/** Renders a heading that visually identifies a related group of fields. */
export function FieldLegend(props: FieldLegendProps): JSX.Element {
  const legend = props.variant !== 'label'
  return <text width="fill" text={props.text} color={props.theme.foreground}
    font_size={legend ? 18 : props.theme.controlFontSize} weight={600}
    role="heading" level={legend ? 2 : 3} />
}

/** Props for a vertical layout that spaces a related group of fields. */
export interface FieldGroupProps {
  /** Palette used to derive the default spacing. */
  theme: Palette
  /** Fields or field sets placed in the group. */
  children: JSX.Element
  /** Gap in logical pixels; defaults to one and a half control paddings. */
  gap?: number
}

/** Arranges related form rows in a consistent vertical stack. */
export function FieldGroup(props: FieldGroupProps): JSX.Element {
  return <column width="fill" gap={props.gap ?? props.theme.controlPadding * 1.5}>{props.children}</column>
}

/** Props for a field row that contains labels, controls, and supporting text. */
export interface FieldProps {
  /** Palette used for state and spacing. */
  theme: Palette
  /** Field descendants. Horizontal fields place siblings side by side. */
  children: JSX.Element
  /** Layout direction. `responsive` currently uses vertical flow on Argui hosts. */
  orientation?: FieldOrientation
  /** Whether this field contains an invalid value. */
  invalid?: boolean
  /** Whether the field's controls are disabled. */
  disabled?: boolean
  /** Whether the field's controls are required. */
  required?: boolean
  /** Spacing between direct field children. */
  gap?: number
}

/** Renders a grouped vertical field or an inline horizontal control-and-label row. */
export function Field(props: FieldProps): JSX.Element {
  const orientation = props.orientation ?? 'vertical'
  const horizontal = orientation === 'horizontal'
  const common = {
    width: 'fill' as const,
    gap: props.gap ?? props.theme.controlPadding,
    role: 'group' as const,
    invalid: !!props.invalid,
    accessible_disabled: !!props.disabled,
    required: !!props.required,
  }
  return horizontal
    ? <row {...common} align_items="center">{props.children}</row>
    : <column {...common}>{props.children}</column>
}

/** Props for the flexible text stack inside a horizontal field. */
export interface FieldContentProps {
  /** Field label and supporting text. */
  children: JSX.Element
  /** Whether the content should grow into remaining horizontal space. */
  grow?: boolean
}

/** Stacks a field's title, label, and description in a flexible column. */
export function FieldContent(props: FieldContentProps): JSX.Element {
  return <column width="fill" min_width={0} grow={props.grow === false ? 0 : 1} gap={5}>{props.children}</column>
}

/** Props for a field label related to a control by native key. */
export interface FieldLabelProps {
  /** Palette used for label text. */
  theme: Palette
  /** Visible and accessible field name. */
  text: string
  /** Control key; the matching label key becomes `${htmlFor}-label`. */
  htmlFor?: string
  /** Optional explicit semantic key, overriding the key derived from `htmlFor`. */
  id?: string
  /** Whether to show a required marker. */
  required?: boolean
  /** Whether to dim the label with a disabled control. */
  disabled?: boolean
}

/** Renders a visible native text label that can be referenced with `labelled_by`. */
export function FieldLabel(props: FieldLabelProps): JSX.Element {
  const key = () => props.id ?? (props.htmlFor ? `${props.htmlFor}-label` : undefined)
  return <row key={key()} role="text" accessible_name={props.text} align_items="center" gap={4}
    opacity={props.disabled ? 0.52 : 1}>
    <text text={props.text} color={props.theme.foreground} font_size={props.theme.controlFontSize}
      weight={500} accessible_hidden={true} />
    {props.required ? <text text="*" color={props.theme.destructive} font_size={props.theme.controlFontSize}
      accessible_hidden={true} /> : null}
  </row>
}

/** Props for a short emphasized title in a field description stack. */
export interface FieldTitleProps {
  /** Palette used for title text. */
  theme: Palette
  /** Short title text. */
  text: string
  /** Whether to dim the title with a disabled field. */
  disabled?: boolean
}

/** Renders a compact title with the same weight as a form label. */
export function FieldTitle(props: FieldTitleProps): JSX.Element {
  return <text width="fill" text={props.text} color={props.disabled ? props.theme.muted : props.theme.foreground}
    font_size={props.theme.controlFontSize} weight={600} />
}

/** Props for supporting copy that can be referenced from a text control. */
export interface FieldDescriptionProps {
  /** Palette used for muted description text. */
  theme: Palette
  /** Supporting instructions. */
  text: string
  /** Optional key for a control's `described_by` relation. */
  id?: string
}

/** Renders muted supporting copy for a field or field set. */
export function FieldDescription(props: FieldDescriptionProps): JSX.Element {
  return <text key={props.id} width="fill" text={props.text} color={props.theme.muted}
    font_size={12} line_height={1.35} />
}

/** Props for a themed rule that can include a short caption. */
export interface FieldSeparatorProps {
  /** Palette used for the rule and caption. */
  theme: Palette
  /** Optional caption displayed between the rule segments. */
  label?: string
  /** Vertical space above and below the rule. */
  margin?: number
}

/** Separates form sections with an optional centered caption. */
export function FieldSeparator(props: FieldSeparatorProps): JSX.Element {
  return <row width="fill" margin_top={props.margin ?? 2} margin_bottom={props.margin ?? 2}
    gap={8} align_items="center" role="separator" orientation="horizontal">
    <rectangle width="fill" height={1} background={props.theme.border} />
    {props.label ? <text text={props.label} color={props.theme.muted} font_size={12} /> : null}
    {props.label ? <rectangle width="fill" height={1} background={props.theme.border} /> : null}
  </row>
}

/** Props for a validation alert associated with a field. */
export interface FieldErrorProps {
  /** Palette used for destructive validation messages. */
  theme: Palette
  /** Optional messages; duplicate message strings are collapsed. */
  errors?: readonly (string | { message?: string } | undefined)[]
  /** Custom native content shown instead of the messages list. */
  children?: JSX.Element
}

/** Announces validation feedback and omits the alert when there is no content. */
export function FieldError(props: FieldErrorProps): JSX.Element {
  const messages = () => uniqueFieldErrorMessages(props.errors)
  return props.children || messages().length > 0
    ? <column role="alert" live="assertive" width="fill" gap={3}>
      {props.children ?? messages().map((message) => <text text={message} color={props.theme.destructive} font_size={12} />)}
    </column>
    : null
}
