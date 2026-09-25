import type { JSX } from '@argui/solid/jsx-runtime'
import { Avatar, AvatarGroup, AvatarGroupCount } from '../../../../packages/widgets/src/solid/avatar'
import { mediaAssets } from '../assets.generated'
import type { Palette } from '@argui/widgets/solid'

/** Demonstrates themed avatar sizes, registered imagery, fallbacks and groups. */
export function AvatarPage(props: { theme: Palette }): JSX.Element {
  return <column width="fill" gap={18}>
    <text width="fill" text="Avatars use registered Argui image assets and show a themed fallback when no image is supplied."
      color={props.theme.muted} font_size={14} />
    <row wrap gap={14} align_items="center">
      <Avatar theme={props.theme} size="sm" fallback="CN" label="Compact fallback avatar" />
      <Avatar theme={props.theme} fallback="EM" label="Default fallback avatar" bordered />
      <Avatar theme={props.theme} size="lg" source={mediaAssets['photo/saturn.jpg']}
        alt="Saturn" label="Online avatar" badge={{ label: 'Online' }} />
    </row>
    <row width="fill" gap={12} align_items="center">
      <text text="Team" color={props.theme.foreground} font_size={14} weight={600} />
      <AvatarGroup label="Project team" overlap>
        <Avatar theme={props.theme} source={mediaAssets['illustration/orbit.png']} alt="Orbit illustration"
          fallback="OR" label="Orbit" size="sm" bordered />
        <Avatar theme={props.theme} source={mediaAssets['photo/saturn.jpg']} alt="Saturn"
          fallback="ST" label="Saturn" size="sm" bordered />
        <Avatar theme={props.theme} fallback="AM" label="Alex Morgan" size="sm" bordered />
        <AvatarGroupCount theme={props.theme} count={4} size="sm" />
      </AvatarGroup>
      <text text="4 more" color={props.theme.muted} font_size={13} />
    </row>
  </column>
}
