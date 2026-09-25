/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactCommand as Command, ReactCommandEmpty as CommandEmpty, ReactCommandGroup as CommandGroup,
  ReactCommandInput as CommandInput, ReactCommandItem as CommandItem, ReactCommandList as CommandList,
  ReactCommandSeparator as CommandSeparator } from '../../../../packages/widgets/src/react/command'

/** Demonstrates grouped searchable commands, disabled entries, shortcut labels, and selected actions. */
export function CommandPage(props: { theme: Palette }): ReactElement {
  const [lastAction, setLastAction] = useState('None selected')
  return <column width="fill" gap={14}>
    <text width="fill" text="Search the command list, move between enabled results with the arrow keys, and press Enter or click to run a command."
      color={props.theme.muted} font_size={13} />
    <Command id="foundation-j-commands" label="Project commands" theme={props.theme} emptyLabel="No command matches that search."
      onSelect={setLastAction}>
      <CommandInput placeholder="Search commands…" />
      <CommandSeparator />
      <CommandList>
        <CommandEmpty />
        <CommandGroup heading="Workspace">
          <CommandItem value="open-file" label="Open file" shortcut="⌘O" />
          <CommandItem value="find-files" label="Find files" shortcut="⌘P" />
          <CommandItem value="close-workspace" label="Close workspace" disabled />
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading="Preferences">
          <CommandItem value="settings" label="Settings" shortcut="⌘," />
          <CommandItem value="keyboard-shortcuts" label="Keyboard shortcuts" shortcut="⌘K" />
        </CommandGroup>
      </CommandList>
    </Command>
    <text text={`Last selected command: ${lastAction}`} color={props.theme.foreground} font_size={13} />
  </column>
}
