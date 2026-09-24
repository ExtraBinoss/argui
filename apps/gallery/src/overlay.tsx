import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { overlayFilters } from './overlay-filters'
import { Popover, type Palette } from '@argui/widgets/solid'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true
const popoverVariants = [
  { id: 'demo-popover', title: 'With backdrop blur', button: 'Open blurred', blur: true, opaque: false,
    detail: 'Colors behind this panel are softened.' },
  { id: 'demo-popover-solid', title: 'Without backdrop blur', button: 'Open unblurred', blur: false, opaque: false,
    detail: 'Colors remain sharp behind this panel.' },
  { id: 'demo-popover-opaque', title: 'Opaque themed surface', button: 'Open opaque', blur: false, opaque: true,
    detail: 'The theme surface fully covers the colors.' },
] as const

/** Compares blurred, unblurred, and opaque anchored popovers in either viewport size. */
export function PopoverPage(props: { theme: Palette }): JSX.Element {
  const [active, setActive] = createSignal<string | null>(null)
  return <column width="fill" gap={14}>
    <text text="All variants use the same themed border and shadow. Click outside or press Escape to dismiss." width="fill"
      color={props.theme.muted} font_size={14} />
    <row width="fill" wrap={true} gap={16}>
      {popoverVariants.map((variant) => <column key={`sample-${variant.id}`} width={mobile ? 'fill' : 310} gap={8}>
        <text text={variant.title} color={props.theme.foreground} font_size={16} weight={600} />
        <rectangle width="fill" height={230} radius={12} background={props.theme.surfaceRaised}>
          <rectangle x={28} y={88} width={112} height={112} radius={56} background="#f97316" />
          <rectangle x={122} y={45} width={132} height={132} radius={66} background="#2563eb" />
          <rectangle x={211} y={116} width={88} height={88} radius={44} background="#e11d48" />
          <column x={16} y={16}>
            <Popover id={variant.id} label={variant.button} blur={variant.blur} opaque={variant.opaque}
              open={active() === variant.id}
              onOpenChange={(open) => setActive((current) => open ? variant.id : current === variant.id ? null : current)}
              theme={props.theme}>
              <text text={variant.title} color={props.theme.foreground} font_size={16} weight={600} />
              <text text={variant.detail} width="fill" color={props.theme.foreground} font_size={13} />
            </Popover>
          </column>
        </rectangle>
      </column>)}
    </row>
  </column>
}

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
