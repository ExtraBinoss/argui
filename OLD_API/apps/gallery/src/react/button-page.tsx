/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import { Button as ReactButton, type Palette } from '@argui/widgets/react'
import { mediaAssets } from '../assets.generated'

/** Keeps button activation state in the shell while demonstrating every variant. */
export function ReactButtonPage(props: { theme: Palette; clicks: number; lastUsed: string; starActive: boolean; activate: (label: string) => void }): ReactElement {
  return (
    <column width="fill" gap={18}>
      <text width="fill" text="Actions, variants and interactive states."
        color={props.theme.muted} font_size={14} />
      <text text="Variants" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <ReactButton id="button-primary" label="Primary" theme={props.theme} kind="primary" onClick={() => props.activate('Primary')} />
        <ReactButton id="button-secondary" label="Secondary" theme={props.theme} kind="secondary" onClick={() => props.activate('Secondary')} />
        <ReactButton id="button-outline" label="Outline" theme={props.theme} kind="outline" onClick={() => props.activate('Outline')} />
        <ReactButton id="button-ghost" label="Ghost" theme={props.theme} kind="ghost" onClick={() => props.activate('Ghost')} />
        <ReactButton id="button-destructive" label="Delete" theme={props.theme} kind="destructive" onClick={() => props.activate('Delete')} />
        <ReactButton id="button-link" label="Link action" theme={props.theme} kind="link" onClick={() => props.activate('Link')} />
      </row>
      <text text="Sizes" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10} align_items="center">
        <ReactButton id="button-xs" label="Extra small" theme={props.theme} size="xs" onClick={() => props.activate('XS')} />
        <ReactButton id="button-sm" label="Small" theme={props.theme} size="sm" onClick={() => props.activate('SM')} />
        <ReactButton id="button-lg" label="Large" theme={props.theme} size="lg" onClick={() => props.activate('LG')} />
        <ReactButton id="button-icon-sm" label="Star small" theme={props.theme} size="icon-sm"
          icon={mediaAssets['tabler/star.svg']} onClick={() => props.activate('Icon SM')} />
      </row>
      <text text="States and icons" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <ReactButton id="button-disabled" label="Disabled" theme={props.theme} kind="outline" disabled onClick={() => props.activate('Disabled')} />
        <ReactButton id="button-busy" label="Loading" theme={props.theme} kind="primary" busy onClick={() => props.activate('Loading')} />
        <ReactButton id="button-star" label="Star" theme={props.theme} kind="outline" selected={props.starActive}
          icon={mediaAssets['tabler/star.svg']} activeIcon={mediaAssets['tabler/star-filled.svg']}
          iconOnly onClick={() => props.activate('Star')} />
      </row>
      <text text={`Clicks: ${props.clicks} · Last used: ${props.lastUsed}`}
        color={props.theme.muted} font_size={13} />
    </column>
  )
}
