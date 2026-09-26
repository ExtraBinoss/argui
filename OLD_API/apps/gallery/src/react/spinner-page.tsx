/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import { Spinner, type Palette } from '@argui/widgets/react'

/** Shows the loading indicator at several sizes with an announced status. */
export function SpinnerPage(props: { theme: Palette }): ReactElement {
  return <column width="fill" gap={16}>
    <text text="The status spinner uses a native rotation loop and follows the current theme."
      width="fill" color={props.theme.muted} font_size={14} />
    <row gap={18} align_items="center">
      <Spinner theme={props.theme} size={12} label="Loading small item" />
      <Spinner theme={props.theme} size={16} label="Loading item" />
      <Spinner theme={props.theme} size={24} label="Loading medium item" />
      <Spinner theme={props.theme} size={32} label="Loading large item" />
    </row>
    <row gap={9} align_items="center">
      <Spinner theme={props.theme} size={18} label="Processing payment" />
      <text text="Processing payment…" color={props.theme.foreground} font_size={14} />
    </row>
  </column>
}
