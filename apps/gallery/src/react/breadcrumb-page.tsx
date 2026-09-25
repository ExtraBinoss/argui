/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import {
  ReactBreadcrumb as Breadcrumb, ReactBreadcrumbEllipsis as BreadcrumbEllipsis,
  ReactBreadcrumbItem as BreadcrumbItem, ReactBreadcrumbLink as BreadcrumbLink,
  ReactBreadcrumbList as BreadcrumbList, ReactBreadcrumbPage as BreadcrumbCurrentPage,
  ReactBreadcrumbSeparator as BreadcrumbSeparator,
} from '../../../../packages/widgets/src/react/breadcrumb'

/** Demonstrates semantic breadcrumb navigation, an omitted path, and application-owned navigation callbacks. */
export function BreadcrumbPage(props: { theme: Palette }): ReactElement {
  const [current, setCurrent] = useState('Component details')
  return <column width="fill" gap={14}>
    <text width="fill" text="Breadcrumb links call back into the application, while the current page and decorative separators expose the appropriate navigation semantics."
      color={props.theme.muted} font_size={13} />
    <Breadcrumb theme={props.theme} label="Documentation breadcrumb">
      <BreadcrumbList theme={props.theme}>
        <BreadcrumbItem theme={props.theme}>
          <BreadcrumbLink id="foundation-i-breadcrumb-home" theme={props.theme} label="Home"
            onNavigate={() => setCurrent('Home')} />
        </BreadcrumbItem>
        <BreadcrumbSeparator theme={props.theme} />
        <BreadcrumbItem theme={props.theme}><BreadcrumbEllipsis theme={props.theme} /></BreadcrumbItem>
        <BreadcrumbSeparator theme={props.theme} text="/" />
        <BreadcrumbItem theme={props.theme}>
          <BreadcrumbLink id="foundation-i-breadcrumb-components" theme={props.theme} label="Components"
            onNavigate={() => setCurrent('Components')} />
        </BreadcrumbItem>
        <BreadcrumbSeparator theme={props.theme} />
        <BreadcrumbItem theme={props.theme}><BreadcrumbCurrentPage theme={props.theme} label={current} /></BreadcrumbItem>
      </BreadcrumbList>
    </Breadcrumb>
    <text text={`Selected destination: ${current}`} color={props.theme.foreground} font_size={13} />
  </column>
}
