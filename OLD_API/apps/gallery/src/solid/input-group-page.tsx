import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextarea } from '../../../../packages/widgets/src/solid/input-group'

/** Shows controlled input-group editing, adornments, an action, and a multiline editor. */
export function InputGroupPage(props: { theme: Palette }): JSX.Element {
  const [query, setQuery] = createSignal('issue:open')
  const [message, setMessage] = createSignal('Summarize the latest release notes.')
  return <column width="fill" gap={14}>
    <text width="fill" text="Input groups combine a native editor with text and actions. Place start adornments before the editor, and use vertical layout for block-aligned content."
      color={props.theme.muted} font_size={13} />
    <InputGroup theme={props.theme} label="Filter issues">
      <InputGroupAddon theme={props.theme} align="inline-start">
        <InputGroupText theme={props.theme} text="Filter" />
      </InputGroupAddon>
      <InputGroupInput id="foundation-h-query" label="Filter issues" theme={props.theme}
        value={query()} onChange={setQuery} placeholder="Search issues" search />
      <InputGroupAddon theme={props.theme} align="inline-end">
        <InputGroupButton id="foundation-h-clear" label="Clear filter" theme={props.theme}
          onClick={() => setQuery('')} size="icon">
          <text text="×" color={props.theme.foreground} font_size={17} />
        </InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
    <text text={`Current filter: ${query() || 'none'}`} color={props.theme.muted} font_size={12} />
    <InputGroup theme={props.theme} label="Release prompt" orientation="vertical">
      <InputGroupTextarea id="foundation-h-prompt" label="Release prompt" theme={props.theme}
        value={message()} onChange={setMessage} height={96} />
      <InputGroupAddon theme={props.theme} align="block-end">
        <InputGroupText theme={props.theme} text={`${message().length} characters`} />
      </InputGroupAddon>
    </InputGroup>
    <InputGroup theme={props.theme} label="Disabled input" disabled>
      <InputGroupInput id="foundation-h-disabled" label="Disabled input" theme={props.theme}
        defaultValue="This control is unavailable" disabled />
    </InputGroup>
  </column>
}
