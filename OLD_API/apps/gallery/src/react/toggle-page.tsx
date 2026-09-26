/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactToggle as Toggle } from '../../../../packages/widgets/src/react/toggle'

/** Shows controlled and uncontrolled toggles across size and border variants. */
export function TogglePage(props: { theme: Palette }): ReactElement {
  const [italic, setItalic] = useState(false)
  const [bookmarked, setBookmarked] = useState(true)
  return <column width="fill" gap={15}>
    <text width="fill" text="Toggles are pressed-state buttons that work with pointer, Enter, Space, and assistive technology."
      color={props.theme.muted} font_size={14} />
    <row wrap gap={10} align_items="center">
      <Toggle id="foundation-g-italic" label="Italic" theme={props.theme} text="Italic"
        pressed={italic} onPressedChange={setItalic} />
      <Toggle id="foundation-g-bookmark" label="Bookmark" theme={props.theme} text="Bookmark"
        variant="outline" pressed={bookmarked} onPressedChange={setBookmarked} />
      <Toggle id="foundation-g-bold-small" label="Bold small" theme={props.theme} text="Bold"
        size="sm" defaultPressed />
      <Toggle id="foundation-g-underline-large" label="Underline large" theme={props.theme} text="Underline"
        size="lg" variant="outline" />
      <Toggle id="foundation-g-disabled-toggle" label="Disabled toggle" theme={props.theme} text="Disabled"
        disabled />
    </row>
  </column>
}
