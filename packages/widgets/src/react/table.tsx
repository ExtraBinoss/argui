/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import {
  foundationKSortRows,
  type FoundationKTableProps,
  type FoundationKTableSort,
} from '../shared/foundation-k'

export type ReactTableProps = FoundationKTableProps
export type TableColumn = import('../shared/foundation-k').FoundationKTableColumn
export type TableRow = import('../shared/foundation-k').FoundationKTableRow
export type TableSort = FoundationKTableSort
export type ReactTableColumn = TableColumn
export type ReactTableRow = TableRow

/** Renders the same accessible, sortable table as the Solid adapter. */
export function ReactTable(props: ReactTableProps): ReactElement {
  const [uncontrolledSort, setUncontrolledSort] = useState<FoundationKTableSort | null>(props.defaultSort ?? null)
  const [uncontrolledSelection, setUncontrolledSelection] = useState<readonly string[]>(props.defaultSelectedRowIds ?? [])
  const sort = props.sort !== undefined ? props.sort : uncontrolledSort
  const selectedIds = props.selectedRowIds ?? uncontrolledSelection
  const mode = props.selectionMode ?? 'none'
  const rows = foundationKSortRows(props.rows, sort)
  const maxHeight = Number.isFinite(props.maxHeight) ? Math.max(120, props.maxHeight!) : 360
  const sortBy = (columnId: string) => {
    if (!props.columns.find((column) => column.id === columnId)?.sortable) return
    const next = sort?.columnId !== columnId
      ? { columnId, direction: 'ascending' as const }
      : sort.direction === 'ascending' ? { columnId, direction: 'descending' as const } : null
    if (props.sort === undefined) setUncontrolledSort(next)
    props.onSortChange?.(next)
  }
  const toggleRow = (rowId: string) => {
    if (mode === 'none') return
    const next = mode === 'single'
      ? selectedIds.includes(rowId) ? [] : [rowId]
      : selectedIds.includes(rowId) ? selectedIds.filter((id) => id !== rowId) : [...selectedIds, rowId]
    if (props.selectedRowIds === undefined) setUncontrolledSelection(next)
    props.onSelectionChange?.(next)
  }
  return <column width="fill" gap={6}>
    {props.label ? <text text={props.label} color={props.theme.foreground} font_size={14} weight={600} /> : null}
    <focusScope nativeKey={`${props.id}-table`} role="table" accessible_name={props.label}
      multiselectable={mode === 'multiple'}>
      <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border}
        border_width={1} radius={props.theme.overlayRadius}>
        <column width="fill" max_height={maxHeight} scroll_y={true}>
          <focusScope nativeKey={`${props.id}-header-row`} role="row" accessible_name="Table headers">
            <rectangle width="fill" background={props.theme.surfaceRaised}>
              <row width="fill" height={42} align_items="center">
                {props.columns.map((column) => {
                  const activeSort = sort?.columnId === column.id ? sort.direction : undefined
                  const canSort = !!column.sortable
                  return <focusScope key={`${props.id}-header-${column.id}`} role="column_header"
                    accessible_name={canSort ? `${column.label}, ${activeSort ?? 'not sorted'}` : column.label}
                    sort={activeSort} focusable={canSort} focus_on_tab_navigation={canSort}
                    keyboard_activation={canSort ? 'enter_or_space' : 'none'}
                    onClick={canSort ? () => sortBy(column.id) : undefined}>
                    <touchArea enabled={canSort} mouse_cursor={canSort ? 'pointer' : 'default'}>
                      <row width={column.width ?? 'fill'} height="fill" gap={4} padding_left={10} padding_right={8} align_items="center">
                        <text text={column.label} color={props.theme.foreground} font_size={12} weight={600} />
                        {activeSort ? <text text={activeSort === 'ascending' ? '↑' : '↓'} color={props.theme.accent}
                          font_size={12} accessible_hidden={true} /> : null}
                      </row>
                    </touchArea>
                  </focusScope>
                })}
              </row>
            </rectangle>
          </focusScope>
          {rows.length === 0
            ? <row width="fill" height={52} padding_left={12} align_items="center">
              <text text={props.emptyLabel ?? 'No rows to display.'} color={props.theme.muted} font_size={13} />
            </row>
            : rows.map((row, rowIndex) => {
              const selected = selectedIds.includes(row.id)
              const accessibleRowName = props.columns.map((column) => `${column.label}: ${String(row.cells[column.id] ?? '')}`).join(', ')
              const interactive = mode !== 'none'
              return <focusScope key={`${props.id}-row-${row.id}`} role="row" accessible_name={accessibleRowName}
                selected={selected} focusable={interactive} focus_on_tab_navigation={interactive}
                keyboard_activation={interactive ? 'enter_or_space' : 'none'}
                onClick={interactive ? () => toggleRow(row.id) : undefined}>
                <touchArea enabled={interactive} mouse_cursor={interactive ? 'pointer' : 'default'}>
                  <rectangle width="fill" height={42} background={selected ? props.theme.surfaceHover : props.theme.surface}
                    border_color={props.theme.border} border_width={1}>
                    <row width="fill" height="fill" align_items="center">
                      {props.columns.map((column) => {
                        const value = row.cells[column.id] ?? ''
                        return <focusScope key={`${props.id}-cell-${row.id}-${column.id}`} role="cell"
                          accessible_name={`${column.label}: ${String(value)}`} focusable={false}>
                          <row width={column.width ?? 'fill'} height="fill" padding_left={10} padding_right={8} align_items="center">
                            <text text={String(value)} color={props.theme.foreground} font_size={12} />
                          </row>
                        </focusScope>
                      })}
                    </row>
                  </rectangle>
                </touchArea>
              </focusScope>
            })}
        </column>
      </rectangle>
    </focusScope>
    {mode !== 'none' ? <text text={`${selectedIds.length} row${selectedIds.length === 1 ? '' : 's'} selected`}
      color={props.theme.muted} font_size={11} role="status" live="polite" /> : null}
  </column>
}
