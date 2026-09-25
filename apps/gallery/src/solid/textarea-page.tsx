import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Textarea, type Palette } from '@argui/widgets/solid'

/** Shows multiline typing, newlines, controlled text, and disabled state. */
export function TextareaPage(props: { theme: Palette }): JSX.Element {
  const [message, setMessage] = createSignal('')
  return <column width="fill" gap={14}>
    <text width="fill" text="Multiline editing supports newline insertion, selection, IME, and scrolling in the native text editor."
      color={props.theme.muted} font_size={14} />
    <Textarea id="foundation-f-message" label="Message" showLabel theme={props.theme} value={message()}
      placeholder="Type your message here." description={`${message().length} characters`}
      required onChange={setMessage} />
    <Textarea id="foundation-f-draft" label="Saved draft" showLabel theme={props.theme}
      defaultValue="This is an uncontrolled draft.\nIt keeps edits in the component." disabled height={100} />
  </column>
}
