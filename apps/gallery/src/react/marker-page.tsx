/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import { Marker, MarkerContent, MarkerIcon, type Palette } from '@argui/widgets/react'

/** Demonstrates the three marker treatments with themed labels and icons. */
export function MarkerPage(props: { theme: Palette }): ReactElement {
  return <column width="fill" gap={22}>
    <text text="Markers label short sections without interrupting the reading order."
      color={props.theme.muted} font_size={14} />
    <Marker theme={props.theme} label="Default marker">
      <MarkerIcon><text text="●" color={props.theme.accent} font_size={14} /></MarkerIcon>
      <MarkerContent><text text="Recent activity" color={props.theme.muted} font_size={14} /></MarkerContent>
    </Marker>
    <Marker theme={props.theme} variant="separator" label="Separator marker">
      <MarkerContent><text text="Today" color={props.theme.muted} font_size={14} /></MarkerContent>
    </Marker>
    <Marker theme={props.theme} variant="border" label="Border marker">
      <MarkerIcon><text text="✦" color={props.theme.accent} font_size={14} /></MarkerIcon>
      <MarkerContent><text text="Pinned items" color={props.theme.muted} font_size={14} /></MarkerContent>
    </Marker>
  </column>
}
