import type { JSX } from '@argui/solid/jsx-runtime'
import { Separator } from '../../../../packages/widgets/src/solid/separator'
import type { Palette } from '@argui/widgets/solid'

/** Demonstrates decorative horizontal and semantic vertical separators. */
export function SeparatorPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={14}>
    <column gap={6}>
      <text text="Native primitives" color={props.theme.foreground} font_size={15} weight={600} />
      <text text="Theme-colored dividers keep content groups distinct."
        color={props.theme.muted} font_size={13} />
    </column>
    <Separator theme={props.theme} />
    <row height={28} gap={12} align_items="center">
      <text text="Overview" color={props.theme.foreground} font_size={13} />
      <Separator theme={props.theme} orientation="vertical" decorative={false} />
      <text text="Activity" color={props.theme.foreground} font_size={13} />
      <Separator theme={props.theme} orientation="vertical" />
      <text text="Settings" color={props.theme.foreground} font_size={13} />
    </row>
  </column>
}
