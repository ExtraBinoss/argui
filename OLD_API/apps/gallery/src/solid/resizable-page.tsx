import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Resizable } from '../../../../packages/widgets/src/solid/resizable'

/** Shows pointer and keyboard resizing between two panels. */
export function ResizablePage(props: { theme: Palette }): JSX.Element {
  const [split, setSplit] = createSignal(50)
  return <column width="fill" gap={14}>
    <text width="fill" text="Drag the divider or focus it and use the arrow keys. Home and End move to the configured limits."
      color={props.theme.muted} font_size={14} />
    <Resizable id="surface-e-resizable" theme={props.theme} value={split()} onValueChange={setSplit}
      width={640} height={260} minValue={20} maxValue={80} handleLabel="Resize navigation and content panels"
      first={<column gap={5} align_items="center">
        <text text="Navigation" color={props.theme.foreground} font_size={16} weight={600} />
        <text text={`${Math.round(split())}%`} color={props.theme.muted} font_size={12} />
      </column>}
      second={<column gap={5} align_items="center">
        <text text="Content" color={props.theme.foreground} font_size={16} weight={600} />
        <text text={`${100 - Math.round(split())}%`} color={props.theme.muted} font_size={12} />
      </column>} />
  </column>
}
