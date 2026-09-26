import { useTheme } from '@argui/solid'
import { Tabs, type WidgetTheme } from '@argui/widgets/solid'

/** Demonstrates default and line tabs, including a disabled tab. */
export function TabsPage() {
  const theme = useTheme<WidgetTheme>()
  const items = [
    { value: 'account', label: 'Account', content: <text color={theme().text}>Manage your account details.</text> },
    { value: 'password', label: 'Password', content: <text color={theme().text}>Update your password.</text> },
    { value: 'billing', label: 'Billing', disabled: true, content: <text color={theme().text}>Billing is unavailable.</text> },
  ]
  return <column width="100%" gap={16}>
    <text color={theme().text} fontSize={24}>Tabs</text>
    <text color={theme().textMuted}>Use arrow keys to change tabs, or Home and End to jump across enabled tabs.</text>
    <Tabs accessibleName="Settings tabs" items={items} />
    <Tabs accessibleName="Settings line tabs" items={items} variant="line" />
  </column>
}
