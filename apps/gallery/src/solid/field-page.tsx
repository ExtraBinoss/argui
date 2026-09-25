import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSet } from '../../../../packages/widgets/src/solid/field'
import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupText } from '../../../../packages/widgets/src/solid/input-group'

/** Demonstrates field composition, semantic label relations, and live validation. */
export function FieldPage(props: { theme: Palette }): JSX.Element {
  const [email, setEmail] = createSignal('')
  const validEmail = () => email().includes('@') && email().includes('.')
  return <column width="fill" gap={14}>
    <text width="fill" text="Compose field sets, labels, descriptions, and validation feedback around native controls. Responsive fields currently use vertical layout on Argui hosts."
      color={props.theme.muted} font_size={13} />
    <FieldSet theme={props.theme} legend="Account details">
      <FieldLegend theme={props.theme} text="Account details" />
      <FieldDescription id="foundation-h-email-help" theme={props.theme}
        text="Use an address you can access to receive sign-in notices." />
      <FieldGroup theme={props.theme}>
        <Field theme={props.theme} invalid={email().length > 0 && !validEmail()} required>
          <FieldLabel htmlFor="foundation-h-email" text="Email address" theme={props.theme} required />
          <InputGroup theme={props.theme} label="Email address" invalid={email().length > 0 && !validEmail()}>
            <InputGroupAddon theme={props.theme} align="inline-start">
              <InputGroupText theme={props.theme} text="@" />
            </InputGroupAddon>
            <InputGroupInput id="foundation-h-email" label="Email address" labelledBy="foundation-h-email-label"
              describedBy="foundation-h-email-help" theme={props.theme} value={email()} onChange={setEmail}
              placeholder="name@example.com" required invalid={email().length > 0 && !validEmail()} />
          </InputGroup>
          <FieldError theme={props.theme} errors={email().length > 0 && !validEmail()
            ? ['Enter an email address with a name and domain.'] : undefined} />
        </Field>
        <Field orientation="horizontal" theme={props.theme}>
          <FieldLabel text="Email notices are optional" theme={props.theme} />
          <FieldDescription theme={props.theme} text="You can change this preference later." />
        </Field>
      </FieldGroup>
    </FieldSet>
  </column>
}
