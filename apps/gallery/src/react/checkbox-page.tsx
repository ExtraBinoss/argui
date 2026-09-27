/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import { Checkbox, type WidgetTheme } from '@argui/widgets/react'
import { mediaAssets } from '../../assets.generated'

/** Demonstrates local, controlled, and disabled checkboxes. */
export function CheckboxPage(): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [updates, setUpdates] = useState(true)
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Checkbox</text>
    <text color={theme.textMuted}>Click or press Space to toggle. Each control announces its checked state.</text>
    <Checkbox checkedIcon={mediaAssets['tabler/check.svg']} accessibleName="Send product updates" label="Send product updates"
      value={updates} onValueChange={setUpdates} />
    <Checkbox checkedIcon={mediaAssets['tabler/check.svg']} accessibleName="Remember this device" label="Remember this device" defaultValue />
    <Checkbox checkedIcon={mediaAssets['tabler/check.svg']} accessibleName="Unavailable option" label="Unavailable option" disabled />
    <text color={theme.textMuted}>{updates ? 'Updates enabled' : 'Updates disabled'}</text>
  </column>
}
