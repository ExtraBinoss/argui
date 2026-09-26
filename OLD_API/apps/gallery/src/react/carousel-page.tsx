/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import { ReactCarousel as Carousel } from '../../../../packages/widgets/src/react/carousel'

/** Shows bounded slide navigation with button, keyboard, and touch input. */
export function CarouselPage(props: { theme: Palette }): ReactElement {
  const names = ['Overview', 'Activity', 'Reports', 'Settings']
  const slides = names.map((name, index) => <column key={`surface-e-slide-${index}`} gap={8} align_items="center">
    <text text={`0${index + 1}`} color={props.theme.accent} font_size={13} weight={700} />
    <text text={name} color={props.theme.foreground} font_size={24} weight={600} />
    <text text="A focused workspace slide with a short description." color={props.theme.muted} font_size={13} />
  </column>)
  return <column width="fill" gap={14}>
    <text width="fill" text="Use Previous and Next, the slide indicators, left and right arrow keys, or swipe horizontally."
      color={props.theme.muted} font_size={14} />
    <Carousel id="surface-e-carousel" theme={props.theme} slides={slides} label="Workspace overview carousel" />
  </column>
}
