import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Sidebar } from '../../../../packages/widgets/src/solid/sidebar'
import type { SurfaceSidebarItem } from '../../../../packages/widgets/src/shared/surface-navigation-sidebar'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates collapse modes, nested navigation, selection and keyboard tree movement. */
export function SidebarPage(props: { theme: Palette }): JSX.Element {
  const [open, setOpen] = createSignal(true)
  const [selected, setSelected] = createSignal('overview')
  const items: readonly SurfaceSidebarItem[] = [
    { id: 'overview', label: 'Overview', icon: '⌂' },
    { id: 'projects', label: 'Projects', icon: '▣', badge: '4', defaultExpanded: true, children: [
      { id: 'project-sites', label: 'Websites' },
      { id: 'project-mobile', label: 'Mobile app' },
      { id: 'project-archive', label: 'Archived' },
    ] },
    { id: 'team', label: 'Team', icon: '◉', children: [
      { id: 'team-members', label: 'Members' },
      { id: 'team-invites', label: 'Invitations', badge: '2' },
    ] },
    { id: 'settings', label: 'Settings', icon: '⚙', children: [
      { id: 'settings-profile', label: 'Profile' },
      { id: 'settings-security', label: 'Security' },
    ] },
    { id: 'billing', label: 'Billing', icon: '$', disabled: true, badge: 'Soon' },
  ]
  return <column width="fill" gap={16}>
    <row width="fill" gap={18} align_items="start">
      <Sidebar id="surface-sidebar" theme={props.theme} label="Workspace navigation"
        heading="Northstar" items={items} collapsible="icon" width={244} height={330}
        open={open()} onOpenChange={setOpen} defaultExpandedItems={{ projects: true }}
        selected={selected()} onSelectedChange={setSelected} />
      <column width="fill" gap={10} padding={12}>
        <text text={`Selected destination: ${selected()}`} color={props.theme.foreground} font_size={16} weight={600} />
        <text width="fill" text={`Sidebar is ${open() ? 'expanded' : 'collapsed to its icon rail'}. Select entries, expand groups, or use Up and Down to move, Right to expand, and Left to collapse.`}
          color={props.theme.muted} font_size={13} />
      </column>
    </row>
    <text width="fill" text="Disabled destinations stay visible to communicate unavailable sections. The sidebar supports a controlled open state and selected route."
      color={props.theme.muted} font_size={13} />
  </column>
}
