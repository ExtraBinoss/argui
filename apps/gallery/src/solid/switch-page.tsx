import { createSignal } from '@argui/solid'
import { useTheme } from '@argui/solid'
import { Switch, type WidgetTheme } from '@argui/widgets/solid'

/** Demonstrates switch sizes and controlled state. */
export function SwitchPage() {
  const theme = useTheme<WidgetTheme>()
  const [notifications, setNotifications] = createSignal(false)
  return <column width="100%" gap={16}>
    <text color={theme().text} fontSize={24}>Switch</text>
    <text color={theme().textMuted}>An immediate on/off setting with native hover, focus, and keyboard activation.</text>
    <Switch accessibleName="Notifications" label="Notifications"
      value={notifications()} onValueChange={setNotifications} />
    <Switch accessibleName="Compact switch" label="Compact switch" size="sm" defaultValue />
    <Switch accessibleName="Disabled switch" label="Disabled switch" disabled />
    <text color={theme().textMuted} text={notifications() ? 'Notifications on' : 'Notifications off'} />
  </column>
}
