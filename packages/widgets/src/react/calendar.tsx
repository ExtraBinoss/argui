/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import {
  surfaceEAddMonths, surfaceECalendarCells, surfaceECalendarKeyDate, surfaceEDateKey,
  surfaceEKey, surfaceEParseDateKey, surfaceEValidDate, surfaceEWithinCalendarBounds,
  type SurfaceECalendarProps,
} from '../shared/surface-e'
import { ReactButton as Button } from './button'

export type ReactCalendarProps = SurfaceECalendarProps

/** Renders a single-date month grid with keyboard date movement and selection. */
export function ReactCalendar(props: ReactCalendarProps): ReactElement {
  const today = surfaceEValidDate(new Date())!
  const initialSelected = surfaceEValidDate(props.value ?? props.defaultValue)
  const initialMonth = surfaceEValidDate(props.month ?? props.defaultMonth ?? initialSelected ?? today)!
  const [localSelected, setLocalSelected] = useState(initialSelected)
  const [localMonth, setLocalMonth] = useState(monthStart(initialMonth))
  const [activeKey, setActiveKey] = useState(surfaceEDateKey(initialSelected ?? initialMonth))
  const [gridFocused, setGridFocused] = useState(false)
  const locale = props.locale ?? 'en-US'
  const monthFormatter = new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric' })
  const fullDateFormatter = new Intl.DateTimeFormat(locale, { dateStyle: 'full' })
  const selectedDate = surfaceEValidDate(props.value ?? localSelected)
  const selectedKey = selectedDate ? surfaceEDateKey(selectedDate) : undefined
  const displayedMonth = monthStart(surfaceEValidDate(props.month ?? localMonth) ?? today)
  const weekStartsOn = Number.isFinite(props.weekStartsOn)
    ? ((Math.trunc(props.weekStartsOn!) % 7) + 7) % 7 : 0
  const cells = surfaceECalendarCells(displayedMonth, weekStartsOn)
  const monthTitle = monthFormatter.format(displayedMonth)
  const canSelect = (date: Date) => surfaceEWithinCalendarBounds(date,
    surfaceEValidDate(props.minDate), surfaceEValidDate(props.maxDate)) && !props.isDateDisabled?.(date)
  const monthChange = (date: Date) => {
    const next = monthStart(date)
    if (next.getFullYear() === displayedMonth.getFullYear() && next.getMonth() === displayedMonth.getMonth()) return
    if (props.month === undefined) setLocalMonth(next)
    props.onMonthChange?.(next)
  }
  const navigateMonth = (amount: number) => {
    const next = surfaceEAddMonths(displayedMonth, amount)
    setActiveKey(surfaceEDateKey(next))
    monthChange(next)
  }
  const activate = (date: Date) => {
    const key = surfaceEDateKey(date)
    const changed = selectedKey !== key
    setActiveKey(key)
    monthChange(date)
    if (!canSelect(date)) return
    if (props.value === undefined) setLocalSelected(date)
    if (changed) props.onValueChange?.(date)
  }
  const onGridKey = (payload: unknown) => {
    const key = surfaceEKey(payload)
    if (!key) return
    const activeDate = surfaceEParseDateKey(activeKey) ?? selectedDate ?? displayedMonth
    if (key === 'Enter' || key === ' ') {
      activate(activeDate)
      return
    }
    const next = surfaceECalendarKeyDate(activeDate, key, weekStartsOn)
    if (!next) return
    const stride = key === 'ArrowLeft' ? -1 : key === 'ArrowRight' ? 1
      : key === 'ArrowUp' ? -7 : key === 'ArrowDown' ? 7
        : key === 'End' || key === 'PageUp' ? -1 : 1
    let candidate = next
    for (let attempts = 0; attempts < 42 && !canSelect(candidate); attempts += 1) {
      candidate = new Date(candidate.getFullYear(), candidate.getMonth(), candidate.getDate() + stride, 12)
    }
    if (canSelect(candidate)) {
      setActiveKey(surfaceEDateKey(candidate))
      monthChange(candidate)
    }
  }
  const days = weekdayLabels(locale, weekStartsOn)
  const gridId = `${props.id}-calendar-grid`
  return <column nativeKey={props.id} width={294} gap={8} role="group" accessible_name={props.label ?? 'Choose date'}>
    <row width="fill" height={34} gap={8} align_items="center" justify_content="space_between">
      <Button id={`${props.id}-previous-month`} label="Previous month" theme={props.theme}
        size="sm" kind="ghost" onClick={() => navigateMonth(-1)} />
      <text text={monthTitle} color={props.theme.foreground} font_size={14} weight={600} />
      <Button id={`${props.id}-next-month`} label="Next month" theme={props.theme}
        size="sm" kind="ghost" onClick={() => navigateMonth(1)} />
    </row>
    <focusScope nativeKey={gridId} role="grid" accessible_name={props.label ?? 'Choose date'}
      accessible_description={monthTitle} focusable={true}
      active_descendant={`${props.id}-cell-${activeKey}`}
      onFocus={() => setGridFocused(true)} onBlur={() => setGridFocused(false)} onKey={onGridKey}>
      <column width="fill" gap={3}>
        <row width="fill" gap={3} role="row">
          {days.map((day) => <column key={`${props.id}-weekday-${day.full}`} nativeKey={`${props.id}-weekday-${day.full}`} width={38} height={24}
            role="column_header" accessible_name={day.full}>
            <row width="fill" height="fill" align_items="center" justify_content="center">
              <text text={day.short} color={props.theme.muted} font_size={11} />
            </row>
          </column>)}
        </row>
        {Array.from({ length: 6 }, (_, weekIndex) => <row key={`${props.id}-week-${weekIndex}`} nativeKey={`${props.id}-week-${weekIndex}`} width="fill" gap={3} role="row">
          {cells.slice(weekIndex * 7, weekIndex * 7 + 7).map((cell) => {
            if (!cell.inMonth && props.showOutsideDays === false) {
              return <rectangle key={`${props.id}-blank-${cell.key}`} nativeKey={`${props.id}-blank-${cell.key}`} width={38} height={36} accessible_hidden={true} />
            }
            const cellKey = `${props.id}-cell-${cell.key}`
            const selected = selectedKey === cell.key
            const active = activeKey === cell.key
            const current = surfaceEDateKey(today) === cell.key
            const disabled = !canSelect(cell.date)
            const label = fullDateFormatter.format(cell.date)
            return <focusScope key={cellKey} nativeKey={cellKey} role="cell" accessible_name={label} selected={selected}
              current={current ? 'date' : undefined} enabled={!disabled} accessible_disabled={disabled}
              focusable={false} keyboard_activation="none" onClick={() => activate(cell.date)}>
              <touchArea width={38} height={36} enabled={!disabled}
                mouse_cursor={disabled ? 'not_allowed' : 'pointer'}>
                <rectangle width="fill" height="fill" radius={props.theme.controlRadius}
                  background={selected ? props.theme.accent : current ? props.theme.surfaceRaised : '#00000000'}
                  border_color={gridFocused && active ? props.theme.foreground : 'transparent'}
                  border_width={gridFocused && active ? 1 : 0} opacity={disabled ? 0.4 : 1}>
                  <row width="fill" height="fill" align_items="center" justify_content="center">
                    <text text={`${cell.date.getDate()}`} color={selected ? props.theme.accentText : props.theme.foreground}
                      font_size={12} />
                  </row>
                </rectangle>
              </touchArea>
            </focusScope>
          })}
        </row>)}
      </column>
    </focusScope>
  </column>
}

function monthStart(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1, 12)
}

function weekdayLabels(locale: string, weekStartsOn: number): Array<{ short: string; full: string }> {
  const formatter = new Intl.DateTimeFormat(locale, { weekday: 'short' })
  const fullFormatter = new Intl.DateTimeFormat(locale, { weekday: 'long' })
  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(2024, 0, 7 + ((weekStartsOn + index) % 7), 12)
    return { short: formatter.format(date), full: fullFormatter.format(date) }
  })
}
