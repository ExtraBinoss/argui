/** @jsxImportSource @argui/react */
import type { ReactElement, ReactNode } from 'react'
import type { Palette } from '../shared/theme'

/** Surface colors used by a message bubble. */
export type BubbleVariant = 'default' | 'secondary' | 'muted' | 'tinted' | 'outline' | 'ghost' | 'destructive'

/** Props for a vertical collection of messages. */
export interface BubbleGroupProps {
  children: ReactNode
  label?: string
}

/** Groups bubbles in reading order with consistent spacing. */
export function BubbleGroup(props: BubbleGroupProps): ReactElement {
  return <column width="fill" gap={12} role="list" accessible_name={props.label}>{props.children}</column>
}

/** Props for a left or right aligned themed message. */
export interface BubbleProps {
  theme: Palette
  content: ReactNode
  reactions?: ReactNode
  variant?: BubbleVariant
  align?: 'start' | 'end'
  label?: string
}

/** Aligns a message body and its optional reactions in a themed surface. */
export function Bubble(props: BubbleProps): ReactElement {
  const variant = props.variant ?? 'default'
  const theme = props.theme
  const background = variant === 'default' ? theme.accent
    : variant === 'secondary' ? theme.surfaceRaised
      : variant === 'muted' ? theme.surfaceHover
        : variant === 'tinted' ? theme.surfaceRaised
          : variant === 'outline' ? theme.background
            : variant === 'destructive' ? `${theme.destructive}26` : '#00000000'
  return <row width="fill" justify_content={props.align === 'end' ? 'end' : 'start'}
    role="list_item" accessible_name={props.label}>
    <column max_width={520} min_width={0} gap={6}>
      <rectangle background={background}
        border_color={variant === 'outline' ? theme.border : '#00000000'}
        border_width={variant === 'outline' ? 1 : 0}
        radius={variant === 'ghost' ? 0 : theme.controlRadius + 3}>
        {props.content}
      </rectangle>
      {props.reactions}
    </column>
  </row>
}

/** Props for the body of a bubble. */
export interface BubbleContentProps {
  children: ReactNode
}

/** Gives caller-composed bubble content consistent native padding. */
export function BubbleContent(props: BubbleContentProps): ReactElement {
  return <column min_width={0} gap={4} padding={12}>{props.children}</column>
}

/** Props for reaction controls composed next to a bubble. */
export interface BubbleReactionsProps {
  theme: Palette
  children: ReactNode
  label?: string
}

/** Places caller-supplied reaction controls in an accessible row. */
export function BubbleReactions(props: BubbleReactionsProps): ReactElement {
  return <row gap={4} padding={3} width="fit" align_items="center"
    background={props.theme.surfaceRaised} radius={props.theme.controlRadius}
    role="group" accessible_name={props.label ?? 'Reactions'}>{props.children}</row>
}
