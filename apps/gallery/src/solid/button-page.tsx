import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, type Palette } from '@argui/widgets/solid'
import { mediaAssets } from '../assets.generated'

/** Shows the native button variants, blocked states, spinner and star toggle. */
export function ButtonPage(props: { theme: Palette; clicks: number; lastUsed: string; starActive: boolean; activate: (label: string) => void }): JSX.Element {
  return (
    <column width="fill" gap={18}>
      <text text="Actions, variants and interactive states." width="fill" color={props.theme.muted} font_size={14} />
      <text text="Variants" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <Button id="button-primary" label="Primary" theme={props.theme} kind="primary" onClick={() => props.activate('Primary')} />
        <Button id="button-secondary" label="Secondary" theme={props.theme} kind="secondary" onClick={() => props.activate('Secondary')} />
        <Button id="button-outline" label="Outline" theme={props.theme} kind="outline" onClick={() => props.activate('Outline')} />
        <Button id="button-ghost" label="Ghost" theme={props.theme} kind="ghost" onClick={() => props.activate('Ghost')} />
        <Button id="button-destructive" label="Delete" theme={props.theme} kind="destructive" onClick={() => props.activate('Delete')} />
        <Button id="button-link" label="Link action" theme={props.theme} kind="link" onClick={() => props.activate('Link')} />
      </row>
      <text text="Sizes" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10} align_items="center">
        <Button id="button-xs" label="Extra small" theme={props.theme} size="xs" onClick={() => props.activate('XS')} />
        <Button id="button-sm" label="Small" theme={props.theme} size="sm" onClick={() => props.activate('SM')} />
        <Button id="button-lg" label="Large" theme={props.theme} size="lg" onClick={() => props.activate('LG')} />
        <Button id="button-icon-sm" label="Star small" theme={props.theme} size="icon-sm"
          icon={mediaAssets['tabler/star.svg']} onClick={() => props.activate('Icon SM')} />
      </row>
      <text text="States and icons" color={props.theme.foreground} font_size={16} weight={600} />
      <row wrap={true} gap={10}>
        <Button id="button-disabled" label="Disabled" theme={props.theme} kind="outline" disabled onClick={() => props.activate('Disabled')} />
        <Button id="button-busy" label="Loading" theme={props.theme} kind="primary" busy onClick={() => props.activate('Loading')} />
        <Button id="button-star" label="Star" theme={props.theme} kind="outline" selected={props.starActive}
          icon={mediaAssets['tabler/star.svg']} activeIcon={mediaAssets['tabler/star-filled.svg']}
          iconOnly onClick={() => props.activate('Star')} />
      </row>
      <text text={`Clicks: ${props.clicks} · Last used: ${props.lastUsed}`}
        color={props.theme.muted} font_size={13} />
    </column>
  )
}
