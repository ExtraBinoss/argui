import type { AssetRef } from '@argui/host'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'

/** Available visual treatments for a native badge. */
export type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'ghost' | 'link'

/** Badge sizing used for regular labels and compact counters. */
export type BadgeSize = 'default' | 'compact'

/** Props for a small, non-interactive label or count. */
export interface BadgeProps {
  /** Resolved application colors and sizing tokens. */
  theme: Palette
  /** Text announced by assistive technology. */
  label: string
  /** Optional decorative icon shown before the label. */
  icon?: AssetRef
  /** Color and border treatment. */
  variant?: BadgeVariant
  /** Compact badges fit short counts and status labels. */
  size?: BadgeSize
}

/** Renders a themed native badge without adding keyboard focus or click behavior. */
export function Badge(props: BadgeProps): JSX.Element {
  const variant = () => props.variant ?? 'default'
  const colors = () => {
    switch (variant()) {
      case 'secondary': return { background: props.theme.surfaceRaised, foreground: props.theme.foreground, border: props.theme.surfaceRaised }
      case 'destructive': return { background: props.theme.destructive, foreground: '#ffffff', border: props.theme.destructive }
      case 'outline': return { background: props.theme.surface, foreground: props.theme.foreground, border: props.theme.border }
      case 'ghost': return { background: '#00000000', foreground: props.theme.foreground, border: '#00000000' }
      case 'link': return { background: '#00000000', foreground: props.theme.accent, border: '#00000000' }
      default: return { background: props.theme.accent, foreground: props.theme.accentText, border: props.theme.accent }
    }
  }
  const size = () => props.size ?? 'default'
  return <rectangle height={size() === 'compact' ? 20 : 24} min_width={size() === 'compact' ? 20 : undefined}
    background={colors().background} border_color={colors().border} border_width={1}
    radius={props.theme.controlRadius} accessible_name={props.label} role="text">
    <row height="fill" padding_left={size() === 'compact' ? 6 : 9}
      padding_right={size() === 'compact' ? 6 : 9} gap={5} align_items="center">
      {props.icon ? <svg source={props.icon} color={colors().foreground} width={12} height={12} /> : null}
      <text text={props.label} color={colors().foreground} font_size={size() === 'compact' ? 11 : 12}
        weight={500} no_wrap={true} />
    </row>
  </rectangle>
}
