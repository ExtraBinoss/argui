/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactToggleGroup as ToggleGroup, ReactToggleGroupItem as ToggleGroupItem } from '../../../../packages/widgets/src/react/toggle-group'

/** Demonstrates controlled single and multiple toggle groups, sizes, disabled items, and arrow-key selection. */
export function ToggleGroupPage(props: { theme: Palette }): ReactElement {
  const [alignment, setAlignment] = useState('left')
  const [formatting, setFormatting] = useState<readonly string[]>(['bold'])
  return <column width="fill" gap={16}>
    <text width="fill" text="Toggle groups keep one tab stop and move their active item with the arrow keys. Enter or Space changes the pressed state."
      color={props.theme.muted} font_size={13} />
    <column width="fill" gap={8}>
      <text text="Text alignment · single selection" color={props.theme.foreground} font_size={13} weight={600} />
      <ToggleGroup id="foundation-i-align" theme={props.theme} label="Text alignment" value={alignment}
        onValueChange={setAlignment} variant="outline">
        <ToggleGroupItem value="left" label="Align left" text="Left" />
        <ToggleGroupItem value="center" label="Align center" text="Center" />
        <ToggleGroupItem value="right" label="Align right" text="Right" />
      </ToggleGroup>
      <text text={`Selected alignment: ${alignment}`} color={props.theme.muted} font_size={12} />
    </column>
    <column width="fill" gap={8}>
      <text text="Formatting · multiple selection" color={props.theme.foreground} font_size={13} weight={600} />
      <ToggleGroup id="foundation-i-format" theme={props.theme} label="Text formatting" type="multiple"
        value={formatting} onValueChange={setFormatting} orientation="horizontal" spacing={6} size="sm">
        <ToggleGroupItem value="bold" label="Bold" text="Bold" />
        <ToggleGroupItem value="italic" label="Italic" text="Italic" />
        <ToggleGroupItem value="underline" label="Underline" text="Underline" />
        <ToggleGroupItem value="strike" label="Strikethrough unavailable" text="Strike" disabled />
      </ToggleGroup>
      <text text={`Selected: ${formatting.join(', ') || 'none'}`} color={props.theme.muted} font_size={12} />
    </column>
  </column>
}
