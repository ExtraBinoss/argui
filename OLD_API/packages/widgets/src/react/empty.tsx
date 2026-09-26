/** @jsxImportSource @argui/react */
import type { AssetRef } from '@argui/host'
import type { ReactElement, ReactNode } from 'react'
import type { Palette } from '../shared/theme'

/** Surface treatment for an empty state. */
export type EmptyVariant = 'default' | 'outline' | 'muted'

/** Props for a centered native empty state. */
export interface EmptyProps {
  /** Resolved application colors and sizing tokens. */
  theme: Palette
  /** Composed empty-state media, copy, and optional actions. */
  children: ReactNode
  /** Optional name for the empty-state group. */
  label?: string
  /** Surface style; default is transparent. */
  variant?: EmptyVariant
  /** Minimum height of the empty-state region. */
  minHeight?: number
  /** Interior spacing around the empty-state content. */
  padding?: number
}

/** Renders a centered status group for a collection with no current content. */
export function ReactEmpty(props: EmptyProps): ReactElement {
  const variant = props.variant ?? 'default'
  return <rectangle width="fill" min_height={props.minHeight ?? 190}
    background={variant === 'muted' ? props.theme.surfaceRaised : '#00000000'}
    border_color={variant === 'outline' ? props.theme.border : '#00000000'}
    border_width={variant === 'outline' ? 1 : 0} radius={props.theme.overlayRadius}
    role="group" accessible_name={props.label}>
    <column width="fill" min_height={props.minHeight ?? 190} gap={props.theme.controlPadding * 2}
      align_items="center" justify_content="center" padding={props.padding ?? props.theme.dialogPadding}>
      {props.children}
    </column>
  </rectangle>
}

/** Props for the centered heading and description region. */
export interface EmptyHeaderProps {
  /** Heading, optional media, and description elements. */
  children: ReactNode
}

/** Groups the media and explanatory text in an empty state. */
export function ReactEmptyHeader(props: EmptyHeaderProps): ReactElement {
  return <column width="fill" max_width={380} gap={8} align_items="center">{props.children}</column>
}

/** Available presentations for empty-state media. */
export type EmptyMediaVariant = 'default' | 'icon'

/** Props for an optional native illustration or icon in an empty state. */
export interface EmptyMediaProps {
  /** Resolved application colors. */
  theme: Palette
  /** Display treatment; icon draws a themed square around the asset. */
  variant?: EmptyMediaVariant
  /** Optional preloaded icon or image asset. */
  source?: AssetRef
  /** Alternative text for meaningful media. */
  alt?: string
  /** Custom native visual shown when no asset source is provided. */
  children?: ReactNode
  /** Icon box size in native layout units. */
  size?: number
}

/** Renders an optional asset or custom visual, hiding decorative media from assistive technology. */
export function ReactEmptyMedia(props: EmptyMediaProps): ReactElement {
  const size = Number.isFinite(props.size) && (props.size ?? 0) > 0 ? props.size! : 42
  const variant = props.variant ?? 'default'
  const accessible = !!props.alt
  const visual = props.source
    ? props.source.kind === 'image'
      ? <image source={props.source} alt={props.alt ?? ''} width={variant === 'icon' ? size - 16 : size}
        height={variant === 'icon' ? size - 16 : size} fit="contain" accessible_hidden={variant === 'icon' || !accessible} />
      : <svg source={props.source} alt={props.alt ?? ''} width={variant === 'icon' ? size - 16 : size}
        height={variant === 'icon' ? size - 16 : size} accessible_hidden={variant === 'icon' || !accessible} />
    : props.children
  if (variant === 'icon') return <rectangle width={size} height={size} radius={props.theme.controlRadius}
    background={props.theme.surfaceRaised} role={accessible ? 'image' : undefined}
    accessible_name={props.alt} accessible_hidden={!accessible}>
    {visual}
  </rectangle>
  return <row min_height={size} justify_content="center" align_items="center"
    role={accessible && !props.source ? 'image' : undefined}
    accessible_name={accessible && !props.source ? props.alt : undefined}
    accessible_hidden={!accessible && !props.source}>{visual}</row>
}

/** Props for the main empty-state heading. */
export interface EmptyTitleProps {
  /** Resolved application colors. */
  theme: Palette
  /** Short heading explaining the empty state. */
  text: string
}

/** Renders an accessible level-two heading for an empty state. */
export function ReactEmptyTitle(props: EmptyTitleProps): ReactElement {
  return <text text={props.text} color={props.theme.foreground} font_size={18}
    weight={600} line_height={1.2} text_align="center" role="heading" level={2} />
}

/** Props for explanatory copy below an empty-state heading. */
export interface EmptyDescriptionProps {
  /** Resolved application colors. */
  theme: Palette
  /** Concise description or next step. */
  text: string
}

/** Renders muted, centered empty-state explanatory text. */
export function ReactEmptyDescription(props: EmptyDescriptionProps): ReactElement {
  return <text width="fill" text={props.text} color={props.theme.muted} font_size={13}
    line_height={1.4} text_align="center" />
}

/** Props for calls to action beneath empty-state copy. */
export interface EmptyContentProps {
  /** Resolved application sizing tokens. */
  theme: Palette
  /** One or more native controls or links. */
  children: ReactNode
}

/** Arranges empty-state actions in a centered row. */
export function ReactEmptyContent(props: EmptyContentProps): ReactElement {
  return <row width="fill" wrap={true} gap={props.theme.controlPadding} justify_content="center"
    align_items="center">{props.children}</row>
}
