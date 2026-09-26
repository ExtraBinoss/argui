/** @jsxImportSource @argui/react */
import type { ReactElement, ReactNode } from 'react'
import type {
  AvatarBadgeProps, AvatarFallbackProps, AvatarGroupCountProps, AvatarGroupProps,
  AvatarImageProps, AvatarProps,
} from '../shared/foundation-b'
import { avatarDimension } from '../shared/foundation-b'

/** Displays an asset avatar or a themed text fallback in the same native frame. */
export function Avatar(props: AvatarProps): ReactElement {
  const size = avatarDimension(props.size)
  return <rectangle width={size} height={size} clip radius={size / 2}
    background={props.theme.surfaceRaised}
    border_color={props.bordered ? props.theme.background : '#00000000'}
    border_width={props.bordered ? 2 : 0}
    role="image" accessible_name={props.label ?? props.alt ?? props.fallback ?? ''}>
    {props.source
      ? <image source={props.source} alt={props.alt ?? props.label ?? ''}
        width="fill" height="fill" fit="cover" accessible_hidden={true} />
      : <row width="fill" height="fill" justify_content="center" align_items="center">
        <text text={props.fallback ?? ''} color={props.theme.muted}
          font_size={size <= 24 ? 10 : 13} weight={600} />
      </row>}
    {props.badge ? <rectangle x={size - (size <= 24 ? 8 : 10)}
      y={size - (size <= 24 ? 8 : 10)} width={size <= 24 ? 8 : 10}
      height={size <= 24 ? 8 : 10} radius={size <= 24 ? 4 : 5}
      background={props.badge.color ?? props.theme.accent}
      border_color={props.theme.background} border_width={2}
      role="status" accessible_name={props.badge.label} /> : null}
  </rectangle>
}

/** Renders an Argui asset cropped to the requested avatar size. */
export function AvatarImage(props: AvatarImageProps): ReactElement {
  const size = avatarDimension(props.size)
  return <rectangle width={size} height={size} clip radius={size / 2}
    background={props.theme.surfaceRaised}>
    <image source={props.source} alt={props.alt} width="fill" height="fill" fit="cover" />
  </rectangle>
}

/** Renders a centered, themed initials or short-name avatar fallback. */
export function AvatarFallback(props: AvatarFallbackProps): ReactElement {
  const size = avatarDimension(props.size)
  return <rectangle width={size} height={size} clip radius={size / 2}
    background={props.theme.surfaceRaised} role="image" accessible_name={props.text}>
    <row width="fill" height="fill" justify_content="center" align_items="center">
      <text text={props.text} color={props.theme.muted}
        font_size={size <= 24 ? 10 : 13} weight={600} />
    </row>
  </rectangle>
}

/** Draws a small accessible status dot using theme colors by default. */
export function AvatarBadge(props: AvatarBadgeProps): ReactElement {
  const requestedSize = props.size ?? 10
  const size = Number.isFinite(requestedSize) && requestedSize > 0 ? requestedSize : 10
  const avatarSize = avatarDimension(props.avatarSize)
  return <rectangle x={avatarSize - size} y={avatarSize - size}
    width={size} height={size} radius={size / 2}
    background={props.color ?? props.theme.accent}
    role={props.label ? 'status' : undefined} accessible_name={props.label}
    accessible_hidden={props.label ? undefined : true} />
}

/** Places avatars in a horizontal group, optionally with negative overlap spacing. */
export function AvatarGroup(props: AvatarGroupProps<ReactNode>): ReactElement {
  const requestedGap = props.gap ?? 8
  const gap = Number.isFinite(requestedGap) ? Math.max(0, requestedGap) : 8
  return <row align_items="center" gap={props.overlap ? -8 : gap}
    role="group" accessible_name={props.label ?? 'Avatar group'}>{props.children}</row>
}

/** Shows the number of additional people in an avatar group. */
export function AvatarGroupCount(props: AvatarGroupCountProps): ReactElement {
  const size = avatarDimension(props.size)
  const count = Number.isFinite(props.count) ? Math.max(0, Math.floor(props.count)) : 0
  const label = props.label ?? `${count} more people`
  return <rectangle width={size} height={size} radius={size / 2}
    background={props.theme.surfaceRaised} border_color={props.theme.background} border_width={2}
    role="text" accessible_name={label}>
    <row width="fill" height="fill" justify_content="center" align_items="center">
      <text text={`+${count}`} color={props.theme.foreground}
        font_size={size <= 24 ? 10 : 12} weight={600} />
    </row>
  </rectangle>
}
