import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Table } from '../../../../packages/widgets/src/solid/table'

const columns = [
  { id: 'invoice', label: 'Invoice', width: 110, sortable: true },
  { id: 'customer', label: 'Customer', sortable: true },
  { id: 'status', label: 'Status', width: 100, sortable: true },
  { id: 'amount', label: 'Amount', width: 90, sortable: true },
]
const rows = [
  { id: 'INV-1042', cells: { invoice: 'INV-1042', customer: 'Amina Yusuf', status: 'Paid', amount: 250 } },
  { id: 'INV-1043', cells: { invoice: 'INV-1043', customer: 'Milo Chen', status: 'Pending', amount: 150 } },
  { id: 'INV-1044', cells: { invoice: 'INV-1044', customer: 'Eden Martin', status: 'Unpaid', amount: 350 } },
  { id: 'INV-1045', cells: { invoice: 'INV-1045', customer: 'Nia Patel', status: 'Paid', amount: 450 } },
]

/** Demonstrates sortable column headers, multi-row selection, and live selection count. */
export function TablePage(props: { theme: Palette }): JSX.Element {
  const [selectedRowIds, setSelectedRowIds] = createSignal<readonly string[]>([])
  return <column width="fill" gap={12}>
    <text text="Activate a sortable column header to cycle through ascending, descending, and unsorted order. Select rows by clicking or pressing Space."
      color={props.theme.muted} font_size={13} />
    <Table id="foundation-k-invoices" label="Recent invoices" theme={props.theme} columns={columns} rows={rows}
      selectionMode="multiple" selectedRowIds={selectedRowIds()} onSelectionChange={setSelectedRowIds} />
    <text text={`Selected invoice ids: ${selectedRowIds().join(', ') || 'none'}`} color={props.theme.foreground} font_size={13} />
  </column>
}
