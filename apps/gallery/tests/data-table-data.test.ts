import { expect, test } from 'bun:test'
import { dataTableView } from '../src/data-table-data'

test('data table filters and sorts the complete dataset before slicing pages', () => {
  const sort = { columnId: 'amount', direction: 'descending' as const }
  const first = dataTableView('', 'Open', sort, 1, 20)
  const second = dataTableView('', 'Open', sort, 2, 20)
  expect(first.count).toBe(900)
  expect(first.pageCount).toBe(45)
  expect(first.rows).toHaveLength(20)
  expect(second.rows).toHaveLength(20)
  expect(Number(first.rows.at(-1)?.cells.amount)).toBeGreaterThanOrEqual(Number(second.rows[0]?.cells.amount))
  expect(new Set([...first.rows, ...second.rows].map((row) => row.id)).size).toBe(40)
})

test('data table searches visible fields and clamps a stale page after filtering', () => {
  const result = dataTableView('ORD-0001', 'All', null, 40, 20)
  expect(result.page).toBe(1)
  expect(result.pageCount).toBe(1)
  expect(result.rows.map((row) => row.id)).toEqual(['order-1'])
  expect(dataTableView('ORD-0001', 'Closed', null, 1, 20).count).toBe(1)
  expect(dataTableView('ORD-0001', 'Open', null, 1, 20).count).toBe(0)
})
