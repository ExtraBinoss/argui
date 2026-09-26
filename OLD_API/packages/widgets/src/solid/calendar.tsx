import { createMemo, createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import {
  surfaceEAddMonths, surfaceECalendarCells, surfaceECalendarKeyDate, surfaceEDateKey,
  surfaceEFormatCalendarDate, surfaceEKey, surfaceEParseDateKey, surfaceEValidDate, surfaceEWithinCalendarBounds,
  type SurfaceECalendarProps,
} from '../shared/surface-e'
import { Button } from './button'
import { Popover } from './popover'
import { useWidgetIcons } from './assets'

export type CalendarProps = SurfaceECalendarProps

/** Renders a single-date month grid with keyboard navigation and month and year dropdowns. */
export function Calendar(props: CalendarProps): JSX.Element {
  const icons = useWidgetIcons()
  const today = surfaceEValidDate(new Date())!
  const initialSelected = surfaceEValidDate(props.value ?? props.defaultValue)
  const initialMonth = surfaceEValidDate(props.month ?? props.defaultMonth ?? initialSelected ?? today)!
  const [localSelected, setLocalSelected] = createSignal(initialSelected)
  const [localMonth, setLocalMonth] = createSignal(monthStart(initialMonth))
  const [activeKey, setActiveKey] = createSignal(surfaceEDateKey(initialSelected ?? initialMonth))
  const [gridFocused, setGridFocused] = createSignal(false)
  const [openPicker, setOpenPicker] = createSignal<'month' | 'year' | null>(null)
  const locale = props.locale ?? 'en-US'
  const selectedDate = () => surfaceEValidDate(props.value ?? localSelected())
  const selectedKey = () => selectedDate() ? surfaceEDateKey(selectedDate()!) : undefined
  const displayedMonth = () => monthStart(surfaceEValidDate(props.month ?? localMonth()) ?? today)
  const weekStartsOn = () => Number.isFinite(props.weekStartsOn)
    ? ((Math.trunc(props.weekStartsOn!) % 7) + 7) % 7 : 0
  const monthCells = () => surfaceECalendarCells(displayedMonth(), weekStartsOn())
  const monthTitle = () => surfaceEFormatCalendarDate(displayedMonth(), locale, 'month')
  const monthName = () => surfaceEFormatCalendarDate(displayedMonth(), locale, 'month-name')
  const years = createMemo(() => {
    const current = displayedMonth().getFullYear()
    const first = surfaceEValidDate(props.minDate)?.getFullYear() ?? current - 100
    const last = surfaceEValidDate(props.maxDate)?.getFullYear() ?? current + 100
    return Array.from({ length: Math.max(0, last - first + 1) }, (_, index) => first + index)
  })
  const monthEnabled = (month: number) => {
    const date = new Date(displayedMonth().getFullYear(), month, 1, 12)
    const min = surfaceEValidDate(props.minDate)
    const max = surfaceEValidDate(props.maxDate)
    return (!min || date.getFullYear() > min.getFullYear()
      || (date.getFullYear() === min.getFullYear() && month >= min.getMonth()))
      && (!max || date.getFullYear() < max.getFullYear()
        || (date.getFullYear() === max.getFullYear() && month <= max.getMonth()))
  }
  const canSelect = (date: Date) => surfaceEWithinCalendarBounds(date,
    surfaceEValidDate(props.minDate), surfaceEValidDate(props.maxDate)) && !props.isDateDisabled?.(date)
  const monthChange = (date: Date) => {
    const next = monthStart(date)
    const previous = displayedMonth()
    if (next.getFullYear() === previous.getFullYear() && next.getMonth() === previous.getMonth()) return
    if (props.month === undefined) setLocalMonth(next)
    props.onMonthChange?.(next)
  }
  const navigateMonth = (amount: number) => {
    const next = surfaceEAddMonths(displayedMonth(), amount)
    setActiveKey(surfaceEDateKey(next))
    monthChange(next)
  }
  const chooseMonth = (month: number) => {
    const next = new Date(displayedMonth().getFullYear(), month, 1, 12)
    setActiveKey(surfaceEDateKey(next))
    monthChange(next)
    setOpenPicker(null)
  }
  const chooseYear = (year: number) => {
    const next = new Date(year, displayedMonth().getMonth(), 1, 12)
    setActiveKey(surfaceEDateKey(next))
    monthChange(next)
    setOpenPicker(null)
  }
  const activate = (date: Date) => {
    const key = surfaceEDateKey(date)
    const changed = selectedKey() !== key
    setActiveKey(key)
    monthChange(date)
    if (!canSelect(date)) return
    if (props.value === undefined) setLocalSelected(date)
    if (changed) props.onValueChange?.(date)
  }
  const onGridKey = (payload: unknown) => {
    const key = surfaceEKey(payload)
    if (!key) return
    const activeDate = surfaceEParseDateKey(activeKey()) ?? selectedDate() ?? displayedMonth()
    if (key === 'Enter' || key === ' ') {
      activate(activeDate)
      return
    }
    const next = surfaceECalendarKeyDate(activeDate, key, weekStartsOn())
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
  const days = weekdayLabels(locale, weekStartsOn())
  const gridId = `${props.id}-calendar-grid`
  const monthPickerId = `${props.id}-month-picker`
  const yearPickerId = `${props.id}-year-picker`
  const cellWidth = 38
  const cellGap = 3
  const gridWidth = 7 * cellWidth + 6 * cellGap
  return <column width={gridWidth} gap={8} role="group" accessible_name={props.label ?? 'Choose date'}>
    <row width="fill" height={34} align_items="center">
      <row width={cellWidth} height="fill" shrink={0} align_items="center" justify_content="center">
        <Button id={`${props.id}-previous-month`}
          label={icons.chevronRight ? 'Previous month' : '‹'} accessibleLabel="Previous month"
          theme={props.theme} icon={icons.chevronRight} iconRotation={180} iconOnly={!!icons.chevronRight}
          size={icons.chevronRight ? 'icon-sm' : 'sm'} kind="ghost" onClick={() => navigateMonth(-1)} />
      </row>
      <container grow={1} min_width={0}>
        <row width="fill" height={34} gap={2} align_items="center" justify_content="center">
          <focusScope key={monthPickerId} role="button" accessible_name={`Choose month, ${monthName()}`}
            expandable={true} expanded={openPicker() === 'month'} controls={`${monthPickerId}-popup`}
            keyboard_activation="enter_or_space" onClick={() => setOpenPicker(openPicker() === 'month' ? null : 'month')}>
            <touchArea mouse_cursor="pointer">
              <rectangle width={90} height={30} radius={props.theme.controlRadius}
                hover_background={props.theme.surfaceHover}>
                <row width="fill" height="fill" gap={3} padding_left={6} padding_right={6} align_items="center">
                  <text text={monthName()} width={60} no_wrap={true} text_overflow="ellipsis"
                    color={props.theme.foreground} font_size={13} weight={600} />
                  {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.foreground} width={14} height={14} /> : null}
                </row>
              </rectangle>
            </touchArea>
          </focusScope>
          <focusScope key={yearPickerId} role="button" accessible_name={`Choose year, ${displayedMonth().getFullYear()}`}
            expandable={true} expanded={openPicker() === 'year'} controls={`${yearPickerId}-popup`}
            keyboard_activation="enter_or_space" onClick={() => setOpenPicker(openPicker() === 'year' ? null : 'year')}>
            <touchArea mouse_cursor="pointer">
              <rectangle width={72} height={30} radius={props.theme.controlRadius}
                hover_background={props.theme.surfaceHover}>
                <row width="fill" height="fill" gap={3} padding_left={5} padding_right={5} align_items="center">
                  <text text={`${displayedMonth().getFullYear()}`} width={38} no_wrap={true} text_overflow="ellipsis"
                    color={props.theme.foreground} font_size={13} weight={600} />
                  {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.foreground} width={14} height={14} /> : null}
                </row>
              </rectangle>
            </touchArea>
          </focusScope>
        </row>
      </container>
      <row width={cellWidth} height="fill" shrink={0} align_items="center" justify_content="center">
        <Button id={`${props.id}-next-month`}
          label={icons.chevronRight ? 'Next month' : '›'} accessibleLabel="Next month"
          theme={props.theme} icon={icons.chevronRight} iconOnly={!!icons.chevronRight}
          size={icons.chevronRight ? 'icon-sm' : 'sm'} kind="ghost" onClick={() => navigateMonth(1)} />
      </row>
    </row>
    <Popover id={monthPickerId} label="Months" theme={props.theme} trigger={false} open={openPicker() === 'month'}
      onOpenChange={(open) => setOpenPicker((current) => open ? 'month' : current === 'month' ? null : current)} width={164} opaque={true}
      contentPadding={4} closeLabel={false} initialFocus={`${monthPickerId}-${displayedMonth().getMonth()}`}>
      <focusScope role="list_box" accessible_name="Months" focusable={false}>
          <column width="fill" max_height={220} scroll_y={true} gap={2}>
            {Array.from({ length: 12 }, (_, month) => <focusScope key={`${monthPickerId}-${month}`} role="option"
              accessible_name={surfaceEFormatCalendarDate(new Date(displayedMonth().getFullYear(), month, 1, 12), locale, 'month-name')}
              selected={month === displayedMonth().getMonth()} enabled={monthEnabled(month)}
              keyboard_activation="enter_or_space" onClick={() => chooseMonth(month)}>
              <touchArea enabled={monthEnabled(month)} mouse_cursor={monthEnabled(month) ? 'pointer' : 'not_allowed'}>
                <rectangle width="fill" height={30} radius={props.theme.controlRadius}
                  background={month === displayedMonth().getMonth() ? props.theme.surfaceRaised : props.theme.surface}
                  hover_background={props.theme.surfaceHover} opacity={monthEnabled(month) ? 1 : 0.45}>
                  <row width="fill" height="fill" padding_left={9} align_items="center">
                    <text text={surfaceEFormatCalendarDate(new Date(displayedMonth().getFullYear(), month, 1, 12), locale, 'month-name')}
                      color={props.theme.foreground} font_size={13} />
                  </row>
                </rectangle>
              </touchArea>
            </focusScope>)}
          </column>
      </focusScope>
    </Popover>
    <Popover id={yearPickerId} label="Years" theme={props.theme} trigger={false} open={openPicker() === 'year'}
      onOpenChange={(open) => setOpenPicker((current) => open ? 'year' : current === 'year' ? null : current)} width={100} opaque={true}
      contentPadding={4} closeLabel={false} initialFocus={`${yearPickerId}-${displayedMonth().getFullYear()}`}
      virtualItems={{ count: years().length, itemHeight: 32, height: 220,
        initialIndex: years().indexOf(displayedMonth().getFullYear()), itemKey: (index) => years()[index]!,
        renderItem: (index) => {
          const year = years()[index]!
          return <focusScope key={`${yearPickerId}-${year}`} role="option" accessible_name={`${year}`}
            selected={year === displayedMonth().getFullYear()} keyboard_activation="enter_or_space"
            onClick={() => chooseYear(year)}>
            <touchArea mouse_cursor="pointer">
              <rectangle width="fill" height={32} radius={props.theme.controlRadius}
                background={year === displayedMonth().getFullYear() ? props.theme.surfaceRaised : props.theme.surface}
                hover_background={props.theme.surfaceHover}>
                <row width="fill" height="fill" padding_left={9} align_items="center">
                  <text text={`${year}`} color={props.theme.foreground} font_size={13} />
                </row>
              </rectangle>
            </touchArea>
          </focusScope>
        } }} />
    <focusScope key={gridId} role="grid" accessible_name={props.label ?? 'Choose date'}
      accessible_description={monthTitle()} focusable={true}
      active_descendant={`${props.id}-cell-${activeKey()}`}
      onFocus={() => setGridFocused(true)} onBlur={() => setGridFocused(false)} onKey={onGridKey}>
      <column width="fill" gap={cellGap}>
        <row width="fill" gap={cellGap} role="row">
          {days.map((day) => <column key={`${props.id}-weekday-${day.full}`} width={cellWidth} height={24}
            role="column_header" accessible_name={day.full}>
            <row width="fill" height="fill" align_items="center" justify_content="center">
              <text text={day.short} color={props.theme.muted} font_size={11} />
            </row>
          </column>)}
        </row>
        {Array.from({ length: 6 }, (_, weekIndex) => <row key={`${props.id}-week-${weekIndex}`} width="fill" gap={cellGap} role="row">
          {monthCells().slice(weekIndex * 7, weekIndex * 7 + 7).map((cell) => {
            if (!cell.inMonth && props.showOutsideDays === false) {
              return <rectangle key={`${props.id}-blank-${cell.key}`} width={cellWidth} height={36} accessible_hidden={true} />
            }
            const cellKey = `${props.id}-cell-${cell.key}`
            const selected = selectedKey() === cell.key
            const active = activeKey() === cell.key
            const current = surfaceEDateKey(today) === cell.key
            const disabled = !canSelect(cell.date)
            const label = surfaceEFormatCalendarDate(cell.date, locale, 'full')
            return <focusScope key={cellKey} role="cell" accessible_name={label} selected={selected}
              current={current ? 'date' : undefined} enabled={!disabled} accessible_disabled={disabled}
              focusable={false} keyboard_activation="none" onClick={() => activate(cell.date)}>
              <touchArea width={cellWidth} height={36} enabled={!disabled}
                mouse_cursor={disabled ? 'not_allowed' : 'pointer'}>
                <rectangle width="fill" height="fill" radius={props.theme.controlRadius}
                  background={selected ? props.theme.accent : '#00000000'}
                  hover_background={disabled ? undefined : selected ? props.theme.accentHover : props.theme.surfaceHover}
                  border_color={gridFocused() && active ? props.theme.foreground : 'transparent'}
                  border_width={gridFocused() && active ? 1 : 0} opacity={disabled ? 0.4 : 1}>
                  <row width="fill" height="fill" align_items="center" justify_content="center">
                    <text text={`${cell.date.getDate()}`}
                      color={selected ? props.theme.accentText : !cell.inMonth ? props.theme.muted
                        : current ? props.theme.accent : props.theme.foreground}
                      font_size={12} weight={current && !selected ? 600 : undefined} />
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
  return Array.from({ length: 7 }, (_, index) => {
    const date = new Date(2024, 0, 7 + ((weekStartsOn + index) % 7), 12)
    return { short: surfaceEFormatCalendarDate(date, locale, 'weekday-short'),
      full: surfaceEFormatCalendarDate(date, locale, 'weekday-long') }
  })
}
