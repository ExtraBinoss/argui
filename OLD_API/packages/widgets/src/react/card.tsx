/** @jsxImportSource @argui/react */
import type { ReactElement, ReactNode } from 'react'
import type { Palette } from '../shared/theme'

/** Props for the raised themed surface that contains a card composition. */
export interface CardProps {
  /** Resolved application colors and sizing tokens. */
  theme: Palette
  /** Native card contents, usually header, content, and footer parts. */
  children: ReactNode
  /** Whether to draw a restrained shadow below the surface. */
  elevated?: boolean
  /** Corner radius override in native layout units. */
  radius?: number
  /** Interior padding override in native layout units. */
  padding?: number
}

/** Renders a themed native card surface with optional elevation. */
export function ReactCard(props: CardProps): ReactElement {
  return <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border}
    border_width={1} radius={props.radius ?? props.theme.overlayRadius}
    shadow_blur={props.elevated === false ? 0 : Math.min(6, props.theme.overlayShadowBlur)}
    shadow_offset_y={props.elevated === false ? 0 : 2} shadow_color={props.theme.overlayShadow}>
    <column width="fill" gap={props.theme.controlPadding * 2}
      padding={props.padding ?? props.theme.dialogPadding}>
      {props.children}
    </column>
  </rectangle>
}

/** Props for the heading row of a card. */
export interface CardHeaderProps {
  /** Resolved application sizing tokens. */
  theme: Palette
  /** Heading and description elements. */
  children: ReactNode
  /** Optional action aligned at the end of the heading row. */
  action?: ReactNode
}

/** Arranges the card title, description, and optional action. */
export function ReactCardHeader(props: CardHeaderProps): ReactElement {
  return <row width="fill" gap={12} align_items="start">
    <column width="fill" min_width={0} grow={1} gap={5}>{props.children}</column>
    {props.action ? <column shrink={0} align_items="end">{props.action}</column> : null}
  </row>
}

/** Props for a semantic card heading. */
export interface CardTitleProps {
  /** Resolved application colors. */
  theme: Palette
  /** Heading shown at the top of the card. */
  text: string
}

/** Renders a card title as a level-three heading. */
export function ReactCardTitle(props: CardTitleProps): ReactElement {
  return <text width="fill" text={props.text} color={props.theme.foreground}
    font_size={18} weight={600} line_height={1.2} role="heading" level={3} />
}

/** Props for subdued descriptive card copy. */
export interface CardDescriptionProps {
  /** Resolved application colors. */
  theme: Palette
  /** Supporting description text. */
  text: string
}

/** Renders the muted description beneath a card title. */
export function ReactCardDescription(props: CardDescriptionProps): ReactElement {
  return <text width="fill" text={props.text} color={props.theme.muted} font_size={13} line_height={1.4} />
}

/** Props for an action slot in a card header. */
export interface CardActionProps {
  /** Action content such as an Argui Button. */
  children: ReactNode
}

/** Aligns card header actions to the trailing edge of their slot. */
export function ReactCardAction(props: CardActionProps): ReactElement {
  return <row shrink={0} justify_content="end" align_items="center">{props.children}</row>
}

/** Props for the main content section of a card. */
export interface CardContentProps {
  /** Resolved application sizing tokens. */
  theme: Palette
  /** Main card content. */
  children: ReactNode
}

/** Groups the main content of a card into a vertical layout. */
export function ReactCardContent(props: CardContentProps): ReactElement {
  return <column width="fill" gap={props.theme.controlPadding}>{props.children}</column>
}

/** Supported horizontal alignment for card footer content. */
export type CardFooterAlignment = 'start' | 'center' | 'end' | 'space_between' | 'space_around' | 'space_evenly'

/** Props for actions or supplemental content at the bottom of a card. */
export interface CardFooterProps {
  /** Resolved application sizing tokens. */
  theme: Palette
  /** Footer controls or labels. */
  children: ReactNode
  /** Horizontal layout for multiple footer children. */
  justify?: CardFooterAlignment
}

/** Places card footer content in a horizontally aligned native row. */
export function ReactCardFooter(props: CardFooterProps): ReactElement {
  return <row width="fill" gap={props.theme.controlPadding} align_items="center" wrap={true}
    justify_content={props.justify ?? 'start'}>{props.children}</row>
}
