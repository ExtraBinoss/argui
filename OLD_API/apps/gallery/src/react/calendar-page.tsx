/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactCalendar as Calendar } from '../../../../packages/widgets/src/react/calendar'
import { surfaceEFormatCalendarDate } from '../../../../packages/widgets/src/shared/surface-e'

/** Shows selectable dates, disabled Sundays, month navigation, and grid keys. */
export function CalendarPage(props: { theme: Palette }): ReactElement {
  const [date, setDate] = useState(new Date())
  const selected = surfaceEFormatCalendarDate(date, 'en-US', 'long')
  return <column width="fill" gap={14}>
    <text width="fill" text="Choose a month or year from the dropdowns. Sundays are disabled."
      color={props.theme.muted} font_size={14} />
    <Calendar id="surface-e-calendar" theme={props.theme} value={date} onValueChange={setDate}
      label="Choose a delivery date" weekStartsOn={1} isDateDisabled={(candidate) => candidate.getDay() === 0} />
    <text text={`Selected date: ${selected}`} color={props.theme.foreground} font_size={13} />
  </column>
}
