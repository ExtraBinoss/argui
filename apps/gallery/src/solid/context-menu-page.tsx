import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { ContextMenu } from '../../../../packages/widgets/src/solid/context-menu'
import type { SurfaceMenuItem } from '../../../../packages/widgets/src/shared/surface-b'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates pointer and keyboard context triggers with functional menu actions. */
export function ContextMenuPage(props: { theme: Palette }): JSX.Element {
  const [lastAction, setLastAction] = createSignal('Nothing selected')
  const items: readonly SurfaceMenuItem[] = [
    { type: 'item', id: 'back', label: 'Back', disabled: true, shortcut: 'Alt ←' },
    { type: 'item', id: 'reload', label: 'Reload', shortcut: '⌘ R', onSelect: () => setLastAction('Reload requested') },
    { type: 'separator', id: 'context-divider' },
    { type: 'checkbox', id: 'toolbar', label: 'Show toolbar', defaultChecked: true },
    { type: 'submenu', id: 'share', label: 'Share', items: [
      { type: 'item', id: 'copy-link', label: 'Copy link', onSelect: () => setLastAction('Link copied') },
      { type: 'item', id: 'send-link', label: 'Send link', onSelect: () => setLastAction('Share dialog opened') },
    ] },
  ]
  return <column width="fill" gap={16}>
    <ContextMenu id="surface-b-context" theme={props.theme} label="Context actions" items={items}
      onSelect={(id) => setLastAction(id)}>
      <rectangle width="fill" height={104} radius={props.theme.controlRadius}
        background={props.theme.surfaceRaised} border_color={props.theme.border} border_width={1}>
        <column width="fill" height="fill" gap={6} padding={16} justify_content="center">
          <text text="Project card" color={props.theme.foreground} font_size={15} weight={600} />
          <text text="Right-click here, or focus and press the Context Menu key / F10." width="fill"
            color={props.theme.muted} font_size={13} />
        </column>
      </rectangle>
    </ContextMenu>
    <text text={`Last action: ${lastAction()}`} color={props.theme.foreground} font_size={14} />
    <text width="fill" text="The menu supports arrow-key movement, Enter or Space selection, check items, submenus and Escape dismissal."
      color={props.theme.muted} font_size={13} />
  </column>
}
