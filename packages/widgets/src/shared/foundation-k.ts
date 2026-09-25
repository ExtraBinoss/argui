import type { Palette } from './theme'

/** One labeled value displayed in the native bar chart. */
export interface FoundationKChartDatum {
  /** Stable identifier used by accessibility and selection callbacks. */
  id: string
  /** Category label shown beside the bar. */
  label: string
  /** Non-negative value scaled against the largest datum. */
  value: number
  /** Optional bar color; the theme accent is used by default. */
  color?: string
}

/** Props shared by Solid and React chart adapters. */
export interface FoundationKChartProps {
  /** Stable native key prefix for chart elements. */
  id: string
  /** Accessible chart name, shown as the visible title. */
  title: string
  /** Optional context shown below the title. */
  description?: string
  /** Theme palette used for bars, labels, and surfaces. */
  theme: Palette
  /** Ordered chart values. Duplicate ids are ignored after their first occurrence. */
  data: readonly FoundationKChartDatum[]
  /** Selected datum id; pass null to clear selection. Omit for internal state. */
  selectedId?: string | null
  /** Initial selection used only when selectedId is omitted. */
  defaultSelectedId?: string
  /** Called after the selection changes. */
  onSelectionChange?: (id: string | null) => void
  /** Formats values for visible labels and accessibility. */
  valueFormatter?: (value: number) => string
  /** Width of the chart in logical pixels. Defaults to 420. */
  width?: number
  /** Height of the plotting region in logical pixels. Defaults to 220. */
  height?: number
}

/** A displayed table column. */
export interface FoundationKTableColumn {
  /** Stable column key used to read each row's cells. */
  id: string
  /** Visible and accessible header text. */
  label: string
  /** Fixed logical width; unspecified columns share remaining space. */
  width?: number
  /** Whether clicking or activating the header sorts by this column. */
  sortable?: boolean
}

/** A data row; each cell is rendered as text. */
export interface FoundationKTableRow {
  /** Stable row id used by selection and accessibility. */
  id: string
  /** Values keyed by FoundationKTableColumn.id. */
  cells: Readonly<Record<string, string | number>>
}

/** Active table sort key and direction. */
export interface FoundationKTableSort {
  /** Column being sorted. */
  columnId: string
  /** Current ordering direction. */
  direction: 'ascending' | 'descending'
}

/** Table selection behavior. */
export type FoundationKTableSelectionMode = 'none' | 'single' | 'multiple'

/** Props shared by Solid and React table adapters. */
export interface FoundationKTableProps {
  /** Stable native key prefix. */
  id: string
  /** Accessible table name, also used as the caption when present. */
  label: string
  /** Theme palette used for the table surface and states. */
  theme: Palette
  /** Visible columns in display order. */
  columns: readonly FoundationKTableColumn[]
  /** Rows to display. */
  rows: readonly FoundationKTableRow[]
  /** Current sort state; omit to use internal state. Pass null for no sort. */
  sort?: FoundationKTableSort | null
  /** Initial sort used only when sort is omitted. */
  defaultSort?: FoundationKTableSort
  /** Called after a sortable header changes the sort order. */
  onSortChange?: (sort: FoundationKTableSort | null) => void
  /** Row selection behavior. Defaults to none. */
  selectionMode?: FoundationKTableSelectionMode
  /** Selected row ids; omit to use internal state. */
  selectedRowIds?: readonly string[]
  /** Initial selection used only when selectedRowIds is omitted. */
  defaultSelectedRowIds?: readonly string[]
  /** Called with the complete selected id list after a row is toggled. */
  onSelectionChange?: (ids: readonly string[]) => void
  /** Height limit for vertical scrolling. Defaults to 360. */
  maxHeight?: number
  /** Message shown when rows is empty. */
  emptyLabel?: string
}

/** Answers stored by questionnaire step id. */
export type FoundationKQuestionnaireAnswer = string | readonly string[]

/** One question in a questionnaire. */
export interface FoundationKQuestionnaireStep {
  /** Stable key for this question and its answer. */
  id: string
  /** Question text, shown as a heading. */
  title: string
  /** Optional explanatory text. */
  description?: string
  /** Input kind; choices are required for single and multiple questions. */
  type: 'single' | 'multiple' | 'text'
  /** Choices available for single or multiple questions. */
  options?: readonly { value: string; label: string; description?: string; disabled?: boolean }[]
  /** Whether an answer is required before moving forward. Defaults to true. */
  required?: boolean
  /** Placeholder for text questions. */
  placeholder?: string
  /** Optional synchronous validator; return an error message to block progress. */
  validate?: (answer: FoundationKQuestionnaireAnswer | undefined) => string | undefined
}

/** Props shared by Solid and React questionnaire adapters. */
export interface FoundationKQuestionnaireProps {
  /** Stable native key prefix. */
  id: string
  /** Accessible name for the questionnaire. */
  label: string
  /** Theme palette used by choices, input, progress, and actions. */
  theme: Palette
  /** Ordered questionnaire steps. Step ids must be unique. */
  steps: readonly FoundationKQuestionnaireStep[]
  /** Current answers; omit to store answers internally. */
  value?: Readonly<Record<string, FoundationKQuestionnaireAnswer>>
  /** Initial answers used only when value is omitted. */
  defaultValue?: Readonly<Record<string, FoundationKQuestionnaireAnswer>>
  /** Called whenever an answer changes. */
  onValueChange?: (value: Readonly<Record<string, FoundationKQuestionnaireAnswer>>) => void
  /** Called after the final step validates successfully. */
  onSubmit?: (value: Readonly<Record<string, FoundationKQuestionnaireAnswer>>) => void
  /** Width of the form in logical pixels. Defaults to 520. */
  width?: number
  /** Labels for navigation actions. */
  previousLabel?: string
  /** Labels for navigation actions. */
  nextLabel?: string
  /** Labels for optional-step navigation. */
  skipLabel?: string
  /** Label shown for the final action. */
  submitLabel?: string
  /** Confirmation shown after a successful submission. */
  completedLabel?: string
}

/** Extracts a keyboard key from the host's event payload. */
export function foundationKKey(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (!payload || typeof payload !== 'object') return undefined
  const event = payload as { key?: unknown; state?: unknown }
  if (event.state !== undefined && event.state !== 'pressed') return undefined
  return typeof event.key === 'string' ? event.key : undefined
}

/** Orders table rows for the provided sort state without mutating the source rows. */
export function foundationKSortRows(
  rows: readonly FoundationKTableRow[],
  sort: FoundationKTableSort | null | undefined,
): FoundationKTableRow[] {
  if (!sort) return [...rows]
  const direction = sort.direction === 'ascending' ? 1 : -1
  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })
  return [...rows].sort((left, right) => {
    const a = left.cells[sort.columnId] ?? ''
    const b = right.cells[sort.columnId] ?? ''
    const comparison = typeof a === 'number' && typeof b === 'number'
      ? a - b
      : collator.compare(String(a), String(b))
    return comparison * direction || left.id.localeCompare(right.id)
  })
}

/** Applies required and custom validation for one questionnaire step. */
export function foundationKValidateQuestion(
  step: FoundationKQuestionnaireStep,
  answer: FoundationKQuestionnaireAnswer | undefined,
): string | undefined {
  const required = step.required !== false
  const empty = answer === undefined || (typeof answer === 'string'
    ? answer.trim().length === 0
    : answer.length === 0)
  if (required && empty) return 'Please answer this question before continuing.'
  return step.validate?.(answer)
}

/** Returns distinct chart items while retaining their original display order. */
export function foundationKChartData(
  data: readonly FoundationKChartDatum[],
): FoundationKChartDatum[] {
  const seen = new Set<string>()
  return data.filter((datum) => {
    if (seen.has(datum.id) || !Number.isFinite(datum.value) || datum.value < 0) return false
    seen.add(datum.id)
    return true
  })
}

/** Chooses a clamped numeric dimension for a chart. */
export function foundationKDimension(value: number | undefined, fallback: number, min: number): number {
  return Number.isFinite(value) ? Math.max(min, value!) : fallback
}
