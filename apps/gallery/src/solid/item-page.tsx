import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Badge, Button, Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemSeparator, ItemTitle, type Palette } from '@argui/widgets/solid'

/** Demonstrates item groups, row variants, media, selection, and footer actions. */
export function ItemPage(props: { theme: Palette }): JSX.Element {
  const [selected, setSelected] = createSignal(false)
  const [refreshes, setRefreshes] = createSignal(0)
  return <column width="fill" gap={16}>
    <text text="Items organize a list row into media, content, and action regions." color={props.theme.muted} font_size={14} />
    <row width="fill" justify_content="space_between" align_items="center">
      <text text="Recent work" color={props.theme.foreground} font_size={16} weight={600} />
      <Badge theme={props.theme} label="2 items" variant="secondary" size="compact" />
    </row>
    <ItemGroup label="Recent work items">
      <Item theme={props.theme} variant="outline" size="sm">
        <ItemHeader>
          <text text="Argui gallery" color={props.theme.foreground} font_size={props.theme.controlFontSize} weight={500} />
          <Badge theme={props.theme} label="Ready" variant="outline" size="compact" />
        </ItemHeader>
        <ItemMedia theme={props.theme} variant="icon">
          <text text="A" color={props.theme.accent} font_size={15} weight={600} />
        </ItemMedia>
        <ItemContent>
          <ItemTitle theme={props.theme} text="Component workspace" />
          <ItemDescription theme={props.theme} text="Build and preview native components." />
        </ItemContent>
      </Item>
      <ItemSeparator theme={props.theme} />
      <Item id="item-select-release" theme={props.theme} label="Select release notes"
        variant="muted" selected={selected()} onClick={() => setSelected(!selected())}>
        <ItemMedia theme={props.theme} variant="icon">
          <text text="R" color={props.theme.foreground} font_size={15} weight={600} />
        </ItemMedia>
        <ItemContent>
          <ItemTitle theme={props.theme} text="Release notes" />
          <ItemDescription theme={props.theme} text="Click or press Enter or Space to select this row." />
        </ItemContent>
        <ItemFooter>
          <text text={selected() ? 'Selected' : 'Click to select'} color={props.theme.muted} font_size={12} />
          <ItemActions><Badge theme={props.theme} label={selected() ? 'Selected' : 'Select'} size="compact" /></ItemActions>
        </ItemFooter>
      </Item>
    </ItemGroup>
    <row width="fill" justify_content="space_between" align_items="center">
      <text text={`Refreshes: ${refreshes()}`} color={props.theme.muted} font_size={12} />
      <Button id="item-refresh" label="Refresh list" theme={props.theme} kind="ghost"
        onClick={() => setRefreshes((count) => count + 1)} />
    </row>
  </column>
}
