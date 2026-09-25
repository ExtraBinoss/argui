/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Menubar } from '../../../../packages/widgets/src/react/menubar'
import type { SurfaceMenubarMenu } from '../../../../packages/widgets/src/shared/surface-b'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates a multi-menu bar, including submenu, checked and radio items. */
export function MenubarPage(props: { theme: Palette }): ReactElement {
  const [lastAction, setLastAction] = useState('Nothing selected')
  const [openMenu, setOpenMenu] = useState<string | undefined>()
  const menus: readonly SurfaceMenubarMenu[] = [
    { id: 'file', label: 'File', items: [
      { type: 'item', id: 'new', label: 'New document', shortcut: '⌘ N', onSelect: () => setLastAction('New document') },
      { type: 'item', id: 'save', label: 'Save', shortcut: '⌘ S', onSelect: () => setLastAction('Saved') },
      { type: 'separator', id: 'file-divider' },
      { type: 'item', id: 'export', label: 'Export…', onSelect: () => setLastAction('Export opened') },
    ] },
    { id: 'edit', label: 'Edit', items: [
      { type: 'item', id: 'undo', label: 'Undo', shortcut: '⌘ Z', onSelect: () => setLastAction('Undo requested') },
      { type: 'item', id: 'disabled-redo', label: 'Redo', disabled: true, shortcut: '⇧ ⌘ Z' },
      { type: 'separator', id: 'edit-divider' },
      { type: 'submenu', id: 'find', label: 'Find', items: [
        { type: 'item', id: 'find-again', label: 'Find again', onSelect: () => setLastAction('Find again') },
      ] },
    ] },
    { id: 'view', label: 'View', items: [
      { type: 'checkbox', id: 'sidebar', label: 'Show sidebar', defaultChecked: true },
      { type: 'group', id: 'zoom', label: 'Zoom', items: [
        { type: 'radio', id: 'zoom-100', label: '100%', group: 'zoom', value: '100' },
        { type: 'radio', id: 'zoom-125', label: '125%', group: 'zoom', value: '125' },
      ] },
    ] },
  ]
  return <column width="fill" gap={16}>
    <Menubar id="surface-b-menubar" theme={props.theme} menus={menus} label="Document commands"
      openMenu={openMenu ?? null} defaultChecked={{ sidebar: true }}
      defaultRadioValues={{ zoom: '100' }} onOpenChange={setOpenMenu} onSelect={setLastAction} />
    <text text={`Open menu: ${openMenu ?? 'none'}`} color={props.theme.muted} font_size={13} />
    <text text={`Last action: ${lastAction}`} color={props.theme.foreground} font_size={14} />
    <text width="fill" text="Focus the menu bar; use Left or Right to change menus, Up or Down to open one, then use the menu's arrow keys. Escape closes the popup."
      color={props.theme.muted} font_size={13} />
  </column>
}
