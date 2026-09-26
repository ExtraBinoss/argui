import type { JSX } from '@argui/solid/jsx-runtime'
import { Skeleton, type Palette } from '@argui/widgets/solid'

/** Shows skeleton shapes and a profile-card loading composition. */
export function SkeletonPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={16}>
    <text text="Pulsing native placeholders for content that is still loading."
      width="fill" color={props.theme.muted} font_size={14} />
    <row width="fill" gap={14} align_items="center">
      <Skeleton theme={props.theme} width={48} height={48} radius={24} />
      <column width="fill" gap={8}>
        <Skeleton theme={props.theme} width={250} height={15} />
        <Skeleton theme={props.theme} width={200} height={15} />
      </column>
    </row>
    <column width={280} gap={10} padding={14} background={props.theme.surfaceRaised}
      radius={props.theme.controlRadius}>
      <Skeleton theme={props.theme} width="fill" height={110} radius={props.theme.controlRadius} />
      <Skeleton theme={props.theme} width="fill" height={14} />
      <Skeleton theme={props.theme} width={205} height={14} />
    </column>
  </column>
}
