import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { DropdownMenu } from '../../../../packages/widgets/src/solid/dropdown-menu'
import type { SurfaceMenuItem } from '../../../../packages/widgets/src/shared/surface-b'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates menu actions, disabled rows, checks, radios, submenus and keyboard navigation. */
export function DropdownMenuPage(props: { theme: Palette }): JSX.Element {
  const [lastAction, setLastAction] = createSignal('Nothing selected')
  const [radioChoice, setRadioChoice] = createSignal('personal')
  const items: readonly SurfaceMenuItem[] = [
    { type: 'label', id: 'file-label', label: 'Workspace' },
    { type: 'item', id: 'new-file', label: 'New file', shortcut: '⌘ N', onSelect: () => setLastAction('New file') },
    { type: 'item', id: 'open-file', label: 'Open…', shortcut: '⌘ O', onSelect: () => setLastAction('Open file') },
    { type: 'item', id: 'disabled-action', label: 'Unavailable action', disabled: true },
    { type: 'separator', id: 'file-divider' },
    { type: 'checkbox', id: 'bookmarks', label: 'Show bookmarks bar' },
    { type: 'group', id: 'profiles', label: 'Open as', items: [
      { type: 'radio', id: 'personal-profile', group: 'profile', value: 'personal', label: 'Personal' },
      { type: 'radio', id: 'work-profile', group: 'profile', value: 'work', label: 'Work' },
    ] },
    { type: 'submenu', id: 'more-tools', label: 'More tools', items: [
      { type: 'item', id: 'extensions', label: 'Extensions', onSelect: () => setLastAction('Extensions') },
      { type: 'item', id: 'task-manager', label: 'Task manager', onSelect: () => setLastAction('Task manager') },
    ] },
  ]
  return <column width="fill" gap={16}>
    <DropdownMenu id="surface-b-dropdown" theme={props.theme} triggerLabel="Open menu"
      label="Workspace actions" items={items} defaultChecked={{ bookmarks: true }}
      defaultRadioValues={{ profile: 'personal' }} onSelect={(id) => setLastAction(id)}
      onRadioValueChange={(_group, value) => setRadioChoice(value)} />
    <text text={`Last action: ${lastAction()}`} color={props.theme.foreground} font_size={14} />
    <text text={`Current profile: ${radioChoice()}`} color={props.theme.muted} font_size={13} />
    <text width="fill" text="Open the menu and use Up or Down, Home or End, type a label to search, and press Enter or Space to activate. Arrow Right opens a submenu; Escape closes the popup."
      color={props.theme.muted} font_size={13} />
  </column>
}
