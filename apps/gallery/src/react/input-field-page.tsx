/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { InputField, type WidgetTheme } from '@argui/widgets/react'
import { mediaAssets } from '../../assets.generated'

/** Shows the controlled value and the search/password input modes. */
export function InputFieldPage(): ReactElement {
  const [name, setName] = useState('Ada Lovelace')
  const [query, setQuery] = useState('')
  const [password, setPassword] = useState('native-secret')
  const [email, setEmail] = useState('not-an-email')
  const [submitted, setSubmitted] = useState(false)
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>InputField</text>
    <text color={theme.textMuted}>Use value and onValueChange for controlled text, or defaultValue for local state.</text>
    <InputField id="input-name" label="Name" value={name} onValueChange={(value) => {
      setName(value)
      setSubmitted(false)
    }} onSubmit={() => setSubmitted(true)} />
    <text color={theme.text} text={`Current value: ${name}`} />
    <text color={theme.text} text={submitted ? `Submitted: ${name}` : 'Press Enter to submit.'} />
    <InputField label="Uncontrolled" defaultValue="Edit without parent state" />
    <InputField label="Search" type="search" value={query} onValueChange={setQuery} placeholder="Search"
      leading={<svg source={mediaAssets['tabler/search.svg']} width={16} height={16} color={theme.textMuted} />} />
    <InputField label="Password" type="password" value={password} onValueChange={setPassword} />
    <InputField label="Email validation" value={email} onValueChange={setEmail} invalid={!email.includes('@')} />
  </column>
}
