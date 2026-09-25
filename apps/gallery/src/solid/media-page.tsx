import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { mediaAssets } from '../assets.generated'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Shows native Tabler SVGs and embedded raster illustrations in both adapters. */
export function MediaPage(props: { theme: Palette }): JSX.Element {
  return (
    <column gap={16}>
      <text text="Imported Tabler SVGs and raster images." width="fill" color={props.theme.muted} font_size={14} />
      <row width="fill" wrap={true} gap={20}>
        <column width={mobile ? 'fill' : 240} gap={8}>
          <text text="Tabler SVG icons" color={props.theme.foreground} font_size={16} />
          <row gap={12}>
            <svg source={mediaAssets['tabler/heart.svg']} color={props.theme.accent} width={36} height={36} />
            <svg source={mediaAssets['tabler/photo.svg']} color={props.theme.accent} width={36} height={36} />
            <svg source={mediaAssets['tabler/player-play.svg']} color={props.theme.accent} width={36} height={36} />
          </row>
        </column>
        <column width={mobile ? 'fill' : 240} height={mobile ? 260 : 200} gap={8}>
          <text text="Multicolor SVG pre-rendered as PNG" color={props.theme.foreground} font_size={16} />
          <image source={mediaAssets['illustration/orbit.png']} alt="Colorful orbit illustration" width="fill" height={160} fit="contain" />
        </column>
        <column width={mobile ? 'fill' : 240} height={mobile ? 260 : 200} gap={8}>
          <text text="Imported JPEG image" color={props.theme.foreground} font_size={16} />
          <image source={mediaAssets['photo/saturn.jpg']} alt="Saturn with its rings" width="fill" height={160} fit="cover" />
        </column>
      </row>
    </column>
  )
}
