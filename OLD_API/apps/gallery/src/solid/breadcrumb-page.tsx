import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { mediaAssets } from '../assets.generated'
import {
  Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage as BreadcrumbCurrentPage, BreadcrumbSeparator,
} from '../../../../packages/widgets/src/solid/breadcrumb'

/** Demonstrates a breadcrumb hierarchy and application-owned navigation callbacks. */
export function BreadcrumbPage(props: { theme: Palette }): JSX.Element {
  const [destination, setDestination] = createSignal<string | null>(null)
  return <column width="fill" gap={14}>
    <text width="fill" text="Muted links lead to the current page; hover a link or activate it to request navigation."
      color={props.theme.muted} font_size={13} />
    <Breadcrumb theme={props.theme} label="Documentation breadcrumb">
      <BreadcrumbList theme={props.theme}>
        <BreadcrumbItem theme={props.theme}>
          <BreadcrumbLink id="foundation-i-breadcrumb-home" theme={props.theme} label="Home"
            onNavigate={() => setDestination('Home')} />
        </BreadcrumbItem>
        <BreadcrumbSeparator theme={props.theme}>
          <svg source={mediaAssets['tabler/chevron-right.svg']} color={props.theme.muted}
            width={14} height={14} accessible_hidden={true} />
        </BreadcrumbSeparator>
        <BreadcrumbItem theme={props.theme}>
          <BreadcrumbLink id="foundation-i-breadcrumb-components" theme={props.theme} label="Components"
            onNavigate={() => setDestination('Components')} />
        </BreadcrumbItem>
        <BreadcrumbSeparator theme={props.theme}>
          <svg source={mediaAssets['tabler/chevron-right.svg']} color={props.theme.muted}
            width={14} height={14} accessible_hidden={true} />
        </BreadcrumbSeparator>
        <BreadcrumbItem theme={props.theme}><BreadcrumbCurrentPage theme={props.theme} label="Breadcrumb" /></BreadcrumbItem>
      </BreadcrumbList>
    </Breadcrumb>
    {destination() ? <text text={`Navigation requested: ${destination()}`} color={props.theme.muted} font_size={13} /> : null}
  </column>
}
