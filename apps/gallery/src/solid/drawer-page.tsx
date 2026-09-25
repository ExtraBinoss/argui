import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Button } from '../../../../packages/widgets/src/solid/button'
import { Drawer } from '../../../../packages/widgets/src/solid/drawer'

/** Demonstrates an adjustable bottom drawer with handle swipe-to-dismiss. */
export function DrawerPage(props: { theme: Palette }): JSX.Element {
  const [goal, setGoal] = createSignal(350)
  const [open, setOpen] = createSignal(false)
  const adjust = (amount: number) => setGoal(Math.max(200, Math.min(400, goal() + amount)))
  return <column width="fill" gap={14}>
    <text width="fill" text="Use the handle to swipe down, press Escape, click the backdrop, or use Close. The panel also works with keyboard focus."
      color={props.theme.muted} font_size={14} />
    <Drawer id="surface-d-drawer" title="Move goal" description="Set your daily activity target."
      triggerLabel="Open drawer" theme={props.theme} side="bottom" height={390}
      open={open()} onOpenChange={setOpen} content={<column width="fill" gap={16}>
        <row width="fill" gap={10} align_items="center" justify_content="center">
          <Button id="surface-d-goal-down" label="Decrease by 10" theme={props.theme}
            kind="outline" disabled={goal() <= 200} onClick={() => adjust(-10)} />
          <column gap={3} align_items="center">
            <text text={`${goal()}`} color={props.theme.foreground} font_size={28} weight={700} />
            <text text="Calories per day" color={props.theme.muted} font_size={12} />
          </column>
          <Button id="surface-d-goal-up" label="Increase by 10" theme={props.theme}
            kind="outline" disabled={goal() >= 400} onClick={() => adjust(10)} />
        </row>
        <Button id="surface-d-goal-save" label="Save goal" theme={props.theme} kind="primary"
          onClick={() => setOpen(false)} />
      </column>} />
  </column>
}
