/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Button, Calendar, InputField, Popover, type Palette } from '@argui/widgets/react'

function dateKey(date: Date): string {
  return `${String(date.getFullYear()).padStart(4, '0')}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

function parseDateKey(value: string): Date | undefined {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value.trim())
  if (!match) return undefined
  const date = new Date(2000, Number(match[2]) - 1, Number(match[3]), 12)
  date.setFullYear(Number(match[1]))
  return dateKey(date) === value.trim() ? date : undefined
}

/** Composes an editable date field with a localized keyboard-ready calendar popup. */
export function DatePickerPage(props: { theme: Palette }): ReactElement {
  const [date, setDate] = useState(() => new Date())
  const [input, setInput] = useState(() => dateKey(new Date()))
  const [invalid, setInvalid] = useState(false)
  const [open, setOpen] = useState(false)
  const [locale, setLocale] = useState<'en-US' | 'fr-FR'>('en-US')
  const formatted = new Intl.DateTimeFormat(locale, { dateStyle: 'long' }).format(date)
  const choose = (next: Date) => {
    setDate(next)
    setInput(dateKey(next))
    setInvalid(false)
    setOpen(false)
  }
  const submit = (value: string) => {
    const parsed = parseDateKey(value)
    if (parsed) choose(parsed)
    else setInvalid(true)
  }
  return <column width="fill" gap={14}>
    <text text="Type a YYYY-MM-DD date and press Enter, or choose one with the calendar keyboard controls."
      color={props.theme.muted} font_size={14} />
    <row gap={8} align_items="center">
      <text text={`Locale: ${locale}`} color={props.theme.muted} font_size={13} />
      <Button id="date-picker-locale" label={locale === 'en-US' ? 'Use French' : 'Use English'}
        theme={props.theme} kind="outline" size="sm"
        onClick={() => setLocale((current) => current === 'en-US' ? 'fr-FR' : 'en-US')} />
    </row>
    <InputField id="date-picker-input" label="Date in YYYY-MM-DD format" theme={props.theme}
      value={input} invalid={invalid} showLabel onChange={(value) => { setInput(value); setInvalid(false) }}
      onSubmit={submit} />
    {invalid ? <text text="Enter a valid date in YYYY-MM-DD format." color={props.theme.destructive}
      font_size={12} role="alert" /> : null}
    <Popover id="date-picker-popup" label={formatted} theme={props.theme} open={open}
      onOpenChange={setOpen} width={340} opaque closeLabel={false}>
      <Calendar id="date-picker-calendar" theme={props.theme} value={date} onValueChange={choose}
        label="Choose a date" locale={locale} weekStartsOn={locale === 'fr-FR' ? 1 : 0} />
    </Popover>
    <text text={`Selected: ${formatted}`} color={props.theme.foreground} font_size={13} />
  </column>
}
