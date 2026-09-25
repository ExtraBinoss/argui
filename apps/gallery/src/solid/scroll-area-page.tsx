import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { ScrollArea } from '../../../../packages/widgets/src/solid/scroll-area'

/** Demonstrates native vertical and horizontal scroll viewports. */
export function ScrollAreaPage(props: { theme: Palette }): JSX.Element {
  const tags = Array.from({ length: 18 }, (_, index) => `v1.2.0-beta.${18 - index}`)
  return <column width="fill" gap={16}>
    <text width="fill" text="Vertical scrolling includes the host scrollbar. Horizontal and two-axis scrolling use the native flickable viewport."
      color={props.theme.muted} font_size={14} />
    <column gap={7}>
      <text text="Tags" color={props.theme.foreground} font_size={14} weight={600} />
      <ScrollArea id="surface-c-vertical-scroll" label="Version tags" theme={props.theme} width={300} height={190}>
        <column width="fill" gap={8}>
          {tags.map((tag) => <column key={`surface-c-tag-${tag}`} gap={8}>
            <text text={tag} color={props.theme.foreground} font_size={13} />
            <rectangle width="fill" height={1} background={props.theme.border} />
          </column>)}
        </column>
      </ScrollArea>
    </column>
    <column gap={7}>
      <text text="Horizontal cards" color={props.theme.foreground} font_size={14} weight={600} />
      <ScrollArea id="surface-c-horizontal-scroll" label="Artwork cards" theme={props.theme}
        width={300} height={150} orientation="horizontal" padding={10}>
        <row gap={10} align_items="center">
          {['Ornella Binni', 'Tom Byrom', 'Vladimir Malyavko', 'Ana Moreira'].map((artist, index) =>
            <column key={`surface-c-art-${artist}`} width={138} gap={7}>
              <rectangle width={138} height={88} radius={8}
                background={index % 2 === 0 ? props.theme.surfaceRaised : props.theme.accent} />
              <text text={`Photo by ${artist}`} color={props.theme.foreground} font_size={11} />
            </column>)}
        </row>
      </ScrollArea>
    </column>
  </column>
}
