import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Popover, type Palette } from '@argui/widgets/solid'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

const popoverVariants = [
  { id: 'demo-popover', title: 'With backdrop blur', button: 'Open blurred', blur: true, opaque: false,
    placement: 'bottom_start',
    detail: 'Colors behind this panel are softened.' },
  { id: 'demo-popover-solid', title: 'Without backdrop blur', button: 'Open unblurred', blur: false, opaque: false,
    placement: 'bottom_end',
    detail: 'Colors remain sharp behind this panel.' },
  { id: 'demo-popover-opaque', title: 'Opaque themed surface', button: 'Open opaque', blur: false, opaque: true,
    placement: 'top_end',
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
              placement={variant.placement} width={260}
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
