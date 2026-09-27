import { createSignal } from '@argui/solid'
import { useTheme } from '@argui/solid'
import { InputField, Popover, type WidgetTheme } from '@argui/widgets/solid'

/** Shows popover surfaces, including a desktop-only outside-window sample. */
export function PopoverPage(props: { desktop: boolean }) {
  const [open, setOpen] = createSignal(false)
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" gap={16}>
    <text color={theme().text} fontSize={24}>Popover</text>
    <text color={theme().textMuted}>Compare opaque, transparent, and blurred surfaces. Click outside or press Escape to dismiss.</text>
    <row gap={8} wrap={true} alignItems="center">
      <Popover id="gallery-popover" trigger="Opaque" opaque={true} contentWidth={240} closeLabel="Done"
        leading={<rectangle width={14} height={14} radii={7} background="oklch(0.6 0.2 250)" />}>
        <text color={theme().text}>The background is fully opaque.</text>
      </Popover>
      <Popover id="gallery-controlled-popover" trigger="Transparent" blur={false} open={open()}
        onOpenChange={setOpen} contentWidth={240} closeLabel="Done"
        leading={<rectangle width={14} height={14} radii={7} background="oklch(0.6 0.2 250 / 45%)" />}>
        <text color={theme().text}>The background is translucent. This popup is controlled by the page.</text>
      </Popover>
      <Popover id="gallery-blurred-popover" trigger="Blurred" blur={true} contentWidth={280} initialFocus="first"
        leading={<rectangle width={14} height={14} radii={7} background="oklch(0.6 0.2 250 / 75%)" />}>
        <InputField label="Search filters" type="search" placeholder="Search options..." />
        <text color={theme().textMuted}>The first control receives focus.</text>
      </Popover>
    </row>
    {props.desktop && <row width="100%" justifyContent="end">
      <Popover id="gallery-outside-popover" trigger="Outside window" placement="rightStart"
        allowOutsideWindow={true} opaque={true} contentWidth={240} closeLabel="Done">
        <text color={theme().text}>This native popup may extend past the window edge.</text>
      </Popover>
    </row>}
  </column>
}
