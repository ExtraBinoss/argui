import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, type Palette } from '@argui/widgets/solid'

/** Shows a composable empty state and a control that fills it with content. */
export function EmptyPage(props: { theme: Palette }): JSX.Element {
  const [created, setCreated] = createSignal(false)
  return <column width="fill" gap={16}>
    <text text="An empty state explains what belongs in a collection and offers a useful next step." color={props.theme.muted} font_size={14} />
    {created() ? <column width="fill" gap={10}>
      <text text="Project created: Argui dashboard" color={props.theme.foreground} font_size={15} />
      <Button id="empty-clear-project" label="Clear project" theme={props.theme} kind="outline"
        onClick={() => setCreated(false)} />
    </column> : <Empty theme={props.theme} variant="outline" label="Projects empty state">
      <EmptyHeader>
        <EmptyMedia theme={props.theme} variant="icon">
          <text text="◇" color={props.theme.accent} font_size={22} weight={600} />
        </EmptyMedia>
        <EmptyTitle theme={props.theme} text="No projects yet" />
        <EmptyDescription theme={props.theme} text="Create a project to keep related work and settings in one place." />
      </EmptyHeader>
      <EmptyContent theme={props.theme}>
        <Button id="empty-create-project" label="Create project" theme={props.theme} kind="primary"
          onClick={() => setCreated(true)} />
      </EmptyContent>
    </Empty>}
    <Empty theme={props.theme} variant="muted" minHeight={120} padding={props.theme.controlPadding} label="Notifications empty state">
      <EmptyHeader>
        <EmptyTitle theme={props.theme} text="You're all caught up" />
        <EmptyDescription theme={props.theme} text="New notifications will appear here." />
      </EmptyHeader>
    </Empty>
  </column>
}
