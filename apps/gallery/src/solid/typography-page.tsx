import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'

/** Demonstrates themed native text hierarchy as a reusable page recipe. */
export function TypographyPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={18}>
    <text text="A clear text hierarchy" color={props.theme.foreground}
      font_size={34} weight={700} line_height={1.12} />
    <text text="Headings, body copy, labels and supporting text use the same live theme tokens."
      color={props.theme.muted} font_size={16} line_height={1.5} />
    <rectangle width="fill" height={1} background={props.theme.border} />
    <text text="Section heading" color={props.theme.foreground}
      font_size={25} weight={650} line_height={1.2} />
    <text text="The page remains readable as the theme switches between light and dark variants."
      color={props.theme.foreground} font_size={15} line_height={1.55} />
    <text text="Subsection heading" color={props.theme.foreground}
      font_size={19} weight={600} line_height={1.3} />
    <text text="Use the muted token for captions and secondary information."
      color={props.theme.muted} font_size={13} line_height={1.4} />
    <rectangle width="fill" background={props.theme.surfaceRaised} radius={props.theme.controlRadius}>
      <column padding={16} gap={6}>
        <text text="“Good typography makes the interface easier to scan.”"
          color={props.theme.foreground} font_size={17} line_height={1.5} />
        <text text="Quotation · theme-aware surface" color={props.theme.muted} font_size={12} />
      </column>
    </rectangle>
    <text text="Small print · 12 px" color={props.theme.muted} font_size={12} />
  </column>
}
