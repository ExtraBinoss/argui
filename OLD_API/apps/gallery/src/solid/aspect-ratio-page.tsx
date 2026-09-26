import type { JSX } from '@argui/solid/jsx-runtime'
import { AspectRatio } from '../../../../packages/widgets/src/solid/aspect-ratio'
import { mediaAssets } from '../assets.generated'
import type { Palette } from '@argui/widgets/solid'

/** Shows 16:9 and square native image frames using Argui's aspect-ratio layout. */
export function AspectRatioPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={16}>
    <text width="fill" text="The native container keeps each frame at its declared ratio while the image fills and crops inside it."
      color={props.theme.muted} font_size={14} />
    <AspectRatio theme={props.theme} ratio={16 / 9} width={460} radius={12}>
      <image source={mediaAssets['photo/saturn.jpg']} alt="Saturn and its rings"
        width="fill" height="fill" fit="cover" />
    </AspectRatio>
    <row wrap gap={14} align_items="center">
      <AspectRatio theme={props.theme} ratio={1} width={140} radius={12}>
        <image source={mediaAssets['illustration/orbit.png']} alt="Colorful orbit illustration"
          width="fill" height="fill" fit="cover" />
      </AspectRatio>
      <text text="Square frame" color={props.theme.muted} font_size={13} />
    </row>
  </column>
}
