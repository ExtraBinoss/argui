import { createMemo, createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import {
  Button, InputField, Pagination, PaginationContent, PaginationItem, PaginationLink,
  PaginationNext, PaginationPrevious, Table, type Palette, type TableSort,
} from '@argui/widgets/solid'
import { dataTableColumns, dataTableRows, dataTableView, type DataTableStatus } from '../data-table-data'

const pageSize = 20
const statuses: readonly DataTableStatus[] = ['All', 'Open', 'Closed']

/** Demonstrates filtering, cross-page sorting, selection, paging, and native table scrolling. */
export function DataTablePage(props: { theme: Palette }): JSX.Element {
  const [query, setQuery] = createSignal('')
  const [status, setStatus] = createSignal<DataTableStatus>('All')
  const [sort, setSort] = createSignal<TableSort | null>({ columnId: 'reference', direction: 'ascending' })
  const [page, setPage] = createSignal(1)
  const [selectedIds, setSelectedIds] = createSignal<readonly string[]>([])
  const view = createMemo(() => dataTableView(query(), status(), sort(), page(), pageSize))
  const pageLinks = () => [...new Set([1, view().page - 1, view().page, view().page + 1, view().pageCount])]
    .filter((value) => value >= 1 && value <= view().pageCount).sort((left, right) => left - right)
  const changeQuery = (value: string) => { setQuery(value); setPage(1) }
  const changeStatus = (value: DataTableStatus) => { setStatus(value); setPage(1) }
  const changeSort = (value: TableSort | null) => { setSort(value); setPage(1) }
  return <column width="fill" gap={14}>
    <text text={`${dataTableRows.length.toLocaleString()} sample orders. Filter, sort every matching row, then page through groups of ${pageSize}. Select rows with a click or keyboard.`}
      color={props.theme.muted} font_size={13} />
    <InputField id="data-table-search" label="Search reference or customer" theme={props.theme}
      value={query()} search showLabel placeholder="Search orders" onChange={changeQuery} />
    <row wrap={true} gap={6} align_items="center">
      <text text="Status" color={props.theme.muted} font_size={12} />
      {statuses.map((option) => <Button id={`data-table-filter-${option.toLowerCase()}`} label={option}
        theme={props.theme} kind="outline" size="sm" selected={status() === option}
        onClick={() => changeStatus(option)} />)}
    </row>
    <Table id="data-table" label="Orders" theme={props.theme} columns={dataTableColumns}
      rows={view().rows} sort={sort()} onSortChange={changeSort} maxHeight={306}
      selectionMode="multiple" selectedRowIds={selectedIds()} onSelectionChange={setSelectedIds}
      emptyLabel="No orders match these filters." />
    <text text={`${view().count} matching orders · ${selectedIds().length} selected across pages · Page ${view().page} of ${view().pageCount}`}
      color={props.theme.muted} font_size={12} role="status" live="polite" />
    <Pagination id="data-table-pagination" label="Order pages" theme={props.theme}
      pageCount={view().pageCount} page={view().page} onPageChange={setPage}>
      <PaginationContent>
        <PaginationItem><PaginationPrevious /></PaginationItem>
        {pageLinks().map((number) => <PaginationItem><PaginationLink page={number} /></PaginationItem>)}
        <PaginationItem><PaginationNext /></PaginationItem>
      </PaginationContent>
    </Pagination>
  </column>
}
