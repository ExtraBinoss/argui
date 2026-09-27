/** @jsxImportSource @argui/react */
import { useTheme } from '@argui/react'
import { Tooltip, type WidgetTheme } from '@argui/widgets/react'

/** Demonstrates the four tooltip surface choices and edge placement. */
export function TooltipPage() {
  const theme = useTheme<WidgetTheme>()
  return <column width="100%" gap={16}>
    <text color={theme.text} fontSize={24}>Tooltip</text>
    <text color={theme.textMuted}>Hover a label to see its tooltip. The last example requests a native surface beyond the window edge.</text>
    <row gap={12} wrap={true} alignItems="center">
      <Tooltip id="gallery-tooltip-opaque" content="An opaque tooltip" opaque={true}>
        <rectangle padding={12} radii={theme.radius} background={theme.secondary}><text color={theme.text}>Opaque</text></rectangle>
      </Tooltip>
      <Tooltip id="gallery-tooltip-blur" content="A blurred tooltip" blur={true}>
        <rectangle padding={12} radii={theme.radius} background={theme.secondary}><text color={theme.text}>Blurred</text></rectangle>
      </Tooltip>
      <Tooltip id="gallery-tooltip-transparent" content="A translucent tooltip" blur={false}>
        <rectangle padding={12} radii={theme.radius} background={theme.secondary}><text color={theme.text}>Transparent</text></rectangle>
      </Tooltip>
    </row>
    <row width="100%" justifyContent="end">
      <Tooltip id="gallery-tooltip-outside" content="This tooltip can extend beyond the window"
        placement="rightStart" allowOutsideWindow={true} opaque={true}>
        <rectangle padding={12} radii={theme.radius} background={theme.secondary}><text color={theme.text}>Outside window</text></rectangle>
      </Tooltip>
    </row>
  </column>
}
