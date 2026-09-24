/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { ReactInputField } from './react-input-field'
import type { Palette } from './theme'

/** Demonstrates the same native input states through the React adapter. */
export function ReactInputsPage(props: { theme: Palette }): ReactElement {
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const [invalidEmail, setInvalidEmail] = useState('invalid@')
  const [search, setSearch] = useState('')
  const [submitted, setSubmitted] = useState(false)
  const invalid = submitted && name.trim().length < 3
  return <column width="fill" gap={16}>
    <text text="Native input fields with desktop and Android keyboard support."
      color={props.theme.muted} font_size={14} />
    <column width={320} gap={14}>
      <ReactInputField id="input-name" label="Name" showLabel theme={props.theme} value={name}
        placeholder="Enter at least 3 characters" invalid={invalid} onChange={(value) => {
          setName(value)
          if (submitted) setSubmitted(false)
        }} onSubmit={() => setSubmitted(true)} />
      <text text={invalid ? 'Enter at least 3 characters.' : submitted ? `Submitted: ${name}` : 'Press Enter to submit.'}
        color={invalid ? props.theme.destructive : props.theme.muted} font_size={12} />
      <ReactInputField id="input-email" label="Email" showLabel theme={props.theme} value={email}
        placeholder="name@example.com" onChange={setEmail} />
      <ReactInputField id="input-search-example" label="Search" showLabel search theme={props.theme}
        value={search} placeholder="Search examples" onChange={setSearch} />
      <ReactInputField id="input-invalid-email" label="Invalid email" showLabel theme={props.theme}
        value={invalidEmail} invalid={!/^[^@]+@[^@]+\.[^@]+$/.test(invalidEmail)}
        placeholder="name@example.com" onChange={setInvalidEmail} />
      <ReactInputField id="input-readonly" label="Read only" showLabel readOnly theme={props.theme}
        value="Read-only value" onChange={() => {}} />
      <ReactInputField id="input-disabled" label="Disabled" showLabel disabled theme={props.theme}
        value="Read only" onChange={() => {}} />
    </column>
  </column>
}
