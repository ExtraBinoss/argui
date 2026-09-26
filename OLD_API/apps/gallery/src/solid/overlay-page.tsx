import type { JSX } from '@argui/solid/jsx-runtime'
import { overlayFilters } from '../overlay-filters'
import { type Palette } from '@argui/widgets/solid'

/** Shows every supported backdrop filter over a colorful native scene. */
export function OverlayPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={16}>
    <text text="Native backdrop filters layered over colored geometry." color={props.theme.muted} font_size={14} />
    <row width="fill" wrap={true} gap={16}>
      {overlayFilters.map((filter) => <column key={`overlay-${filter.name}`} gap={7}>
        <text text={filter.name} color={props.theme.foreground} font_size={15} weight={600} />
        <rectangle width={220} height={138} radius={12} clip={true} background={props.theme.surfaceRaised}>
          <rectangle x={-8} y={12} width={94} height={94} radius={47} background="#f97316" />
          <rectangle x={64} y={-10} width={116} height={116} radius={58} background="#2563eb" />
          <rectangle x={146} y={46} width={94} height={94} radius={47} background="#e11d48" />
          <rectangle x={23} y={25} width={174} height={86} radius={10}
            background="#ffffff66" backdrop_filter={filter.value}>
            <column width="fill" height="fill" padding={12} gap={5}>
              <text text="Overlay" color="#101116" font_size={17} weight={600} />
              <text text="Native effect" color="#242838" font_size={12} />
            </column>
          </rectangle>
        </rectangle>
        <text text={filter.value} color={props.theme.muted} font_size={11} />
      </column>)}
    </row>
  </column>
}
