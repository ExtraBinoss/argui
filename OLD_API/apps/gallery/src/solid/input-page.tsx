import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { InputField, type Palette } from '@argui/widgets/solid'

/** Demonstrates native editing, search, disabled state and submit validation. */
export function InputsPage(props: { theme: Palette }): JSX.Element {
  const [name, setName] = createSignal('')
  const [email, setEmail] = createSignal('')
  const [invalidEmail, setInvalidEmail] = createSignal('invalid@')
  const [search, setSearch] = createSignal('')
  const [password, setPassword] = createSignal('')
  const [submitted, setSubmitted] = createSignal(false)
  const invalid = () => submitted() && name().trim().length < 3
  return <column width="fill" gap={16}>
    <text text="Native input fields with desktop and Android keyboard support."
      color={props.theme.muted} font_size={14} />
    <column width={320} gap={14}>
      <InputField id="input-name" label="Name" showLabel theme={props.theme} value={name()}
        placeholder="Enter at least 3 characters" invalid={invalid()} onChange={(value) => {
          setName(value)
          if (submitted()) setSubmitted(false)
        }} onSubmit={() => setSubmitted(true)} />
      <text text={invalid() ? 'Enter at least 3 characters.' : submitted() ? `Submitted: ${name()}` : 'Press Enter to submit.'}
        color={invalid() ? props.theme.destructive : props.theme.muted} font_size={12} />
      <InputField id="input-email" label="Email" showLabel theme={props.theme} value={email()}
        placeholder="name@example.com" onChange={setEmail} />
      <InputField id="input-default" label="Uncontrolled" showLabel theme={props.theme}
        defaultValue="Edit this default value" />
      <InputField id="input-search-example" label="Search" showLabel search theme={props.theme}
        value={search()} placeholder="Search examples" onChange={setSearch} />
      <InputField id="input-password" label="Password" showLabel password theme={props.theme}
        value={password()} placeholder="Enter a password" onChange={setPassword} />
      <InputField id="input-invalid-email" label="Invalid email" showLabel theme={props.theme}
        value={invalidEmail()} invalid={!/^[^@]+@[^@]+\.[^@]+$/.test(invalidEmail())}
        placeholder="name@example.com" onChange={setInvalidEmail} />
      <InputField id="input-readonly" label="Read only" showLabel readOnly theme={props.theme}
        value="Read-only value" onChange={() => {}} />
      <InputField id="input-disabled" label="Disabled" showLabel disabled theme={props.theme}
        value="Read only" onChange={() => {}} />
    </column>
  </column>
}
