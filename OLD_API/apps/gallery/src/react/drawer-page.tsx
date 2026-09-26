/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactButton as Button } from '../../../../packages/widgets/src/react/button'
import { ReactDrawer as Drawer } from '../../../../packages/widgets/src/react/drawer'

/** Demonstrates an adjustable bottom drawer with handle swipe-to-dismiss. */
export function DrawerPage(props: { theme: Palette }): ReactElement {
  const [goal, setGoal] = useState(350)
  const [open, setOpen] = useState(false)
  const adjust = (amount: number) => setGoal((current) => Math.max(200, Math.min(400, current + amount)))
  return <column width="fill" gap={14}>
    <text width="fill" text="Use the handle to swipe down, press Escape, click the backdrop, or use Close. The panel also works with keyboard focus."
      color={props.theme.muted} font_size={14} />
    <Drawer id="surface-d-drawer" title="Move goal" description="Set your daily activity target."
      triggerLabel="Open drawer" theme={props.theme} side="bottom" height={390}
      open={open} onOpenChange={setOpen} content={<column width="fill" gap={16}>
        <row width="fill" gap={10} align_items="center" justify_content="center">
          <Button id="surface-d-goal-down" label="Decrease by 10" theme={props.theme}
            kind="outline" disabled={goal <= 200} onClick={() => adjust(-10)} />
          <column gap={3} align_items="center">
            <text text={`${goal}`} color={props.theme.foreground} font_size={28} weight={700} />
            <text text="Calories per day" color={props.theme.muted} font_size={12} />
          </column>
          <Button id="surface-d-goal-up" label="Increase by 10" theme={props.theme}
            kind="outline" disabled={goal >= 400} onClick={() => adjust(10)} />
        </row>
        <Button id="surface-d-goal-save" label="Save goal" theme={props.theme} kind="primary"
          onClick={() => setOpen(false)} />
      </column>} />
  </column>
}
