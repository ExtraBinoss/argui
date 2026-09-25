import { foundationKSortRows, type FoundationKTableRow, type FoundationKTableSort } from '../../../packages/widgets/src/shared/foundation-k'

/** The fixed columns used by both Data Table gallery adapters. */
export const dataTableColumns = [
  { id: 'reference', label: 'Reference', width: 112, sortable: true },
  { id: 'customer', label: 'Customer', width: 180, sortable: true },
  { id: 'status', label: 'Status', width: 100, sortable: true },
  { id: 'amount', label: 'Amount', width: 100, sortable: true },
] as const

const customers = ['Avery Stone', 'Camille Martin', 'Jordan Lee', 'Morgan Chen', 'Robin Taylor', 'Samira Diallo', 'Taylor Reed', 'Yuki Tanaka']

/** A deterministic 1,200-row dataset for exercising the paginated table. */
export const dataTableRows: readonly FoundationKTableRow[] = Array.from({ length: 1200 }, (_, index) => ({
  id: `order-${index + 1}`,
  cells: {
    reference: `ORD-${String(index + 1).padStart(4, '0')}`,
    customer: customers[index % customers.length]!,
    status: index % 4 === 0 ? 'Closed' : 'Open',
    amount: ((index * 37) % 900) + 50,
  },
}))

export type DataTableStatus = 'All' | 'Open' | 'Closed'

/** Filters, sorts, and then pages the complete dataset so ordering crosses page boundaries. */
export function dataTableView(
  query: string,
  status: DataTableStatus,
  sort: FoundationKTableSort | null,
  requestedPage: number,
  pageSize: number,
): { rows: FoundationKTableRow[]; count: number; page: number; pageCount: number } {
  const needle = query.trim().toLocaleLowerCase()
  const filtered = dataTableRows.filter((row) => {
    if (status !== 'All' && row.cells.status !== status) return false
    return !needle || String(row.cells.reference).toLocaleLowerCase().includes(needle)
      || String(row.cells.customer).toLocaleLowerCase().includes(needle)
  })
  const pageCount = Math.max(1, Math.ceil(filtered.length / pageSize))
  const page = Math.min(pageCount, Math.max(1, requestedPage))
  return {
    rows: foundationKSortRows(filtered, sort).slice((page - 1) * pageSize, page * pageSize),
    count: filtered.length,
    page,
    pageCount,
  }
}
