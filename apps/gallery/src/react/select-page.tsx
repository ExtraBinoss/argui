/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Select, type SelectOption, type WidgetTheme } from '@argui/widgets/react'
import { mediaAssets } from '../../assets.generated'

const languages: SelectOption[] = [
  { value: 'rust', label: 'Rust' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'zig', label: 'Zig', disabled: true },
  { value: 'swift', label: 'Swift' },
]

const fruits: SelectOption[] = [
  { value: 'apple', label: 'Apple' },
  { value: 'banana', label: 'Banana' },
  { value: 'blueberry', label: 'Blueberry' },
  { value: 'grapes', label: 'Grapes' },
  { value: 'pineapple', label: 'Pineapple' },
]

/** Shows standard and shadcn selects with controlled values and native option lists. */
export function SelectPage(): ReactElement {
  const [language, setLanguage] = useState('rust')
  const [fruit, setFruit] = useState('')
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Select</text>
    <text color={theme.textMuted}>Selection is controlled with value and onValueChange.</text>
    <Select id="language-select" label="Language" options={languages} value={language} onValueChange={setLanguage}
      trailing={<svg source={mediaAssets['tabler/chevron-down.svg']} width={16} height={16} color={theme.textMuted} />} />
    <text color={theme.text} text={`Selected value: ${language}`} />
    <Select label="Local selection" options={languages} defaultValue="typescript" />
    <text color={theme.text} fontSize={18}>Shadcn variant</text>
    <text color={theme.textMuted}>The group title appears in the menu. Choose a fruit to update the trigger.</text>
    <Select id="fruit-select" variant="shadcn" label="Fruits" placeholder="Select a fruit"
      width={192} options={fruits} value={fruit} onValueChange={setFruit}
      trailing={<svg source={mediaAssets['tabler/chevron-down.svg']} width={16} height={16} color={theme.textMuted} />} />
    <text color={theme.text} text={`Selected fruit: ${fruit || 'none'}`} />
  </column>
}
