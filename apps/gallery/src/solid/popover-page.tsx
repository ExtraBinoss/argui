import { createSignal } from '@argui/solid'
import { useTheme } from '@argui/solid'
import { InputField, Popover, type WidgetTheme } from '@argui/widgets/solid'

/** Shows a trigger anchored to content in a native popup window. */
export function PopoverPage() {
  const [open, setOpen] = createSignal(false)
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" gap={16}>
    <text color={theme().text} fontSize={24}>Popover</text>
    <text color={theme().textMuted}>Open and dismiss state belongs to the widget unless open is supplied.</text>
    <Popover id="gallery-popover" trigger="Show details" placement="bottomStart" closeLabel="Done">
      <text color={theme().text} fontSize={18}>Popover content</text>
      <text color={theme().textMuted}>The popup is positioned by native layout and closes on outside click or Escape.</text>
    </Popover>
    <Popover id="gallery-controlled-popover" trigger="Controlled details" open={open()}
      onOpenChange={setOpen} closeLabel="Done">
      <text color={theme().text}>Controlled open state follows the parent.</text>
    </Popover>
    <text color={theme().textMuted}>The trigger keeps its natural width while contentWidth sets the popup width.</text>
    <Popover trigger="Filters" contentWidth={320} initialFocus="first" accessibleLabel="Search filters">
      <InputField label="Search filters" type="search" placeholder="Search options..." />
      <text color={theme().textMuted}>The first control receives focus. Tab can move outside this nonmodal popup.</text>
    </Popover>
  </column>
}
