/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Textarea, type Palette } from '@argui/widgets/react'

/** Shows multiline typing, newlines, controlled text, and disabled state. */
export function TextareaPage(props: { theme: Palette }): ReactElement {
  const [message, setMessage] = useState('')
  return <column width="fill" gap={14}>
    <text width="fill" text="Multiline editing supports newline insertion, selection, IME, and scrolling in the native text editor."
      color={props.theme.muted} font_size={14} />
    <Textarea id="foundation-f-message" label="Message" showLabel theme={props.theme} value={message}
      placeholder="Type your message here." description={`${message.length} characters`}
      required onChange={setMessage} />
    <Textarea id="foundation-f-draft" label="Saved draft" showLabel theme={props.theme}
      defaultValue="This is an uncontrolled draft.\nIt keeps edits in the component." disabled height={100} />
  </column>
}
