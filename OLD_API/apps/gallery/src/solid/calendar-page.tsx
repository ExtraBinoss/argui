import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Calendar } from '../../../../packages/widgets/src/solid/calendar'
import { surfaceEFormatCalendarDate } from '../../../../packages/widgets/src/shared/surface-e'

/** Shows selectable dates, disabled Sundays, month navigation, and grid keys. */
export function CalendarPage(props: { theme: Palette }): JSX.Element {
  const [date, setDate] = createSignal(new Date())
  const selected = () => surfaceEFormatCalendarDate(date(), 'en-US', 'long')
  return <column width="fill" gap={14}>
    <text width="fill" text="Choose a month or year from the dropdowns. Sundays are disabled."
      color={props.theme.muted} font_size={14} />
    <Calendar id="surface-e-calendar" theme={props.theme} value={date()} onValueChange={setDate}
      label="Choose a delivery date" weekStartsOn={1} isDateDisabled={(candidate) => candidate.getDay() === 0} />
    <text text={`Selected date: ${selected()}`} color={props.theme.foreground} font_size={13} />
  </column>
}
