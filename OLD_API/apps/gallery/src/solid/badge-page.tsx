import type { JSX } from '@argui/solid/jsx-runtime'
import { type Palette } from '@argui/widgets/solid'
import { Badge } from '../../../../packages/widgets/src/solid/badge'

/** Shows each badge treatment alongside compact notification counts. */
export function BadgePage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={14}>
    <text text="Badges label categories and short counts without taking keyboard focus." color={props.theme.muted} font_size={14} />
    <row width="fill" wrap={true} gap={8}>
      <Badge theme={props.theme} label="Default" />
      <Badge theme={props.theme} label="Secondary" variant="secondary" />
      <Badge theme={props.theme} label="Destructive" variant="destructive" />
      <Badge theme={props.theme} label="Outline" variant="outline" />
      <Badge theme={props.theme} label="Ghost" variant="ghost" />
      <Badge theme={props.theme} label="Link" variant="link" />
    </row>
    <row width="fill" wrap={true} gap={8} align_items="center">
      <text text="Unread" color={props.theme.foreground} font_size={14} />
      <Badge theme={props.theme} label="8" size="compact" />
      <Badge theme={props.theme} label="99+" size="compact" variant="destructive" />
      <Badge theme={props.theme} label="Ready" size="compact" variant="outline" />
    </row>
  </column>
}
