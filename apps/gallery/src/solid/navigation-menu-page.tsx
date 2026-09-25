import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { NavigationMenu } from '../../../../packages/widgets/src/solid/navigation-menu'
import type { SurfaceNavigationItem } from '../../../../packages/widgets/src/shared/surface-navigation-sidebar'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates popup navigation panels and direct destinations. */
export function NavigationMenuPage(props: { theme: Palette }): JSX.Element {
  const [openMenu, setOpenMenu] = createSignal<string | undefined>()
  const [selected, setSelected] = createSignal('home')
  const [lastVisited, setLastVisited] = createSignal('/home')
  const items: readonly SurfaceNavigationItem[] = [
    { id: 'home', label: 'Home', href: '/home' },
    { id: 'docs', label: 'Docs', links: [
      { id: 'docs-intro', label: 'Introduction', description: 'Start with the core concepts and setup.', href: '/docs' },
      { id: 'docs-install', label: 'Installation', description: 'Install the native widget packages.', href: '/docs/install' },
      { id: 'docs-accessibility', label: 'Accessibility', description: 'Keyboard and screen reader behavior.', href: '/docs/accessibility' },
    ] },
    { id: 'components', label: 'Components', links: [
      { id: 'component-menus', label: 'Menus', description: 'Dropdown menus and navigation surfaces.', href: '/components/menus' },
      { id: 'component-forms', label: 'Forms', description: 'Inputs, validation, and selection controls.', href: '/components/forms' },
      { id: 'component-overlays', label: 'Overlays', description: 'Dialogs, popovers, and tooltips.', href: '/components/overlays' },
    ] },
    { id: 'disabled', label: 'Coming soon', disabled: true, links: [] },
  ]
  return <column width="fill" gap={18}>
    <NavigationMenu id="surface-nav-menu" theme={props.theme} label="Documentation navigation"
      items={items} value={openMenu() ?? null} onValueChange={setOpenMenu}
      selected={selected()} onSelectedChange={setSelected} onSelect={(_id, href) => setLastVisited(href ?? '')} />
    <text text={`Open panel: ${openMenu() ?? 'none'}`} color={props.theme.muted} font_size={13} />
    <text text={`Current destination: ${lastVisited()}`} color={props.theme.foreground} font_size={14} />
    <text width="fill" text="Use Left or Right to move between destinations. Down opens a panel; use Up or Down to select a link, then Enter or Space to navigate. Escape closes the panel."
      color={props.theme.muted} font_size={13} />
  </column>
}
