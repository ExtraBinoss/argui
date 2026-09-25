import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Sheet } from '../../../../packages/widgets/src/solid/sheet'

/** Demonstrates sheets attached to each supported viewport edge. */
export function SheetPage(props: { theme: Palette }): JSX.Element {
  const sides = ['top', 'right', 'bottom', 'left'] as const
  return <column width="fill" gap={14}>
    <text width="fill" text="Each sheet traps modal focus, restores it to its trigger, and closes from its close button, Escape, or the backdrop."
      color={props.theme.muted} font_size={14} />
    <text width="fill" text="Side placement is immediate; the host does not currently animate panels in from an edge."
      color={props.theme.muted} font_size={12} />
    <row width="fill" wrap={true} gap={10}>
      {sides.map((side) => <Sheet id={`surface-d-sheet-${side}`} title={`Edit profile from ${side}`}
        description="Update profile details in a modal side panel." triggerLabel={side} side={side}
        theme={props.theme} content={<column gap={8}>
          <text text="Name" color={props.theme.muted} font_size={12} />
          <text text="Pedro Duarte" color={props.theme.foreground} font_size={14} />
          <text text="Username" color={props.theme.muted} font_size={12} />
          <text text="@peduarte" color={props.theme.foreground} font_size={14} />
        </column>} />)}
    </row>
  </column>
}
