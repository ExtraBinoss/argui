/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '../../../../packages/widgets/src/react/collapsible'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates controlled and default-open collapsible panels. */
export function CollapsiblePage(props: { theme: Palette }): ReactElement {
  const [open, setOpen] = useState(false)
  return <column width="fill" gap={20}>
    <text text={`Controlled panel: ${open ? 'open' : 'closed'}`} color={props.theme.muted} font_size={13} />
    <Collapsible id="surface-a-collapsible" theme={props.theme} open={open} onOpenChange={setOpen}
      label="Repository details">
      <CollapsibleTrigger>Show repository details</CollapsibleTrigger>
      <CollapsibleContent>
        <column gap={8}>
          <text text="@argui/primitives" color={props.theme.foreground} font_size={14} />
          <text text="@argui/widgets" color={props.theme.foreground} font_size={14} />
          <text text="@argui/gallery" color={props.theme.foreground} font_size={14} />
        </column>
      </CollapsibleContent>
    </Collapsible>
    <Collapsible id="surface-a-collapsible-default" theme={props.theme} defaultOpen label="Keyboard note">
      <CollapsibleTrigger>Default-open example</CollapsibleTrigger>
      <CollapsibleContent>
        <text width="fill" text="The trigger responds to Enter and Space and announces the expanded state." color={props.theme.foreground} font_size={14} />
      </CollapsibleContent>
    </Collapsible>
  </column>
}
