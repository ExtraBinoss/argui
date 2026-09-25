/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactPagination as Pagination, ReactPaginationContent as PaginationContent,
  ReactPaginationEllipsis as PaginationEllipsis, ReactPaginationItem as PaginationItem,
  ReactPaginationLink as PaginationLink, ReactPaginationNext as PaginationNext,
  ReactPaginationPrevious as PaginationPrevious } from '../../../../packages/widgets/src/react/pagination'

const projects = ['Argui', 'Argui Gallery', 'Widget Catalog', 'Theme Runtime', 'Native Host', 'Web Host', 'CLI Generator', 'Accessibility Bridge', 'Example App', 'Component Registry', 'Release Notes', 'Roadmap']

/** Demonstrates controlled page changes, disabled edge actions, page links, and a range ellipsis. */
export function PaginationPage(props: { theme: Palette }): ReactElement {
  const [page, setPage] = useState(3)
  const pageSize = 2
  const pageCount = Math.ceil(projects.length / pageSize)
  const visible = projects.slice((page - 1) * pageSize, page * pageSize)
  return <column width="fill" gap={14}>
    <text width="fill" text="Page links and previous/next actions share controlled page state. The host exposes navigation as callbacks; the application decides whether to update a route or data query."
      color={props.theme.muted} font_size={13} />
    <column width="fill" gap={5}>
      {visible.map((name, index) => <row key={`foundation-j-project-${index}`} width="fill" height={38} padding_left={10} padding_right={10}
        align_items="center" background={props.theme.surface}>
        <text text={name} color={props.theme.foreground} font_size={13} />
      </row>)}
    </column>
    <Pagination id="foundation-j-pagination" label="Project pages" theme={props.theme} pageCount={pageCount}
      page={page} onPageChange={setPage}>
      <PaginationContent>
        <PaginationItem><PaginationPrevious /></PaginationItem>
        <PaginationItem><PaginationLink page={1} /></PaginationItem>
        <PaginationItem><PaginationLink page={2} /></PaginationItem>
        <PaginationItem><PaginationLink page={3} /></PaginationItem>
        <PaginationItem><PaginationEllipsis /></PaginationItem>
        <PaginationItem><PaginationLink page={pageCount} /></PaginationItem>
        <PaginationItem><PaginationNext /></PaginationItem>
      </PaginationContent>
    </Pagination>
    <text text={`Page ${page} of ${pageCount} · ${visible.length} projects shown`} color={props.theme.muted} font_size={12} />
  </column>
}
