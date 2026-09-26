import type { JSX } from '@argui/solid/jsx-runtime'
import type { MessageAvatarProps as SharedMessageAvatarProps, MessageContentProps as SharedMessageContentProps,
  MessageFooterProps as SharedMessageFooterProps, MessageGroupProps as SharedMessageGroupProps,
  MessageHeaderProps as SharedMessageHeaderProps, MessageProps as SharedMessageProps } from '../shared/chat-components'
import { Avatar } from './avatar'

export type MessageProps = SharedMessageProps<JSX.Element>
export type MessageGroupProps = SharedMessageGroupProps<JSX.Element>
export type MessageContentProps = SharedMessageContentProps<JSX.Element>
export type MessageHeaderProps = SharedMessageHeaderProps<JSX.Element>
export type MessageFooterProps = SharedMessageFooterProps<JSX.Element>
export type MessageAvatarProps = SharedMessageAvatarProps

/** Groups message rows in reading order with an accessible conversation name. */
export function MessageGroup(props: MessageGroupProps): JSX.Element {
  const gap = Number.isFinite(props.gap) && (props.gap ?? -1) >= 0 ? props.gap! : 14
  return <column width="fill" gap={gap} role="list" accessible_name={props.label ?? 'Messages'}>
    {props.children}
  </column>
}

/** Aligns one accessible message row and optional sender avatar. */
export function Message(props: MessageProps): JSX.Element {
  const align = props.align ?? 'start'
  return <row key={props.id} width="fill" gap={10} align_items="end"
    justify_content={align === 'end' ? 'end' : 'start'} role="list_item" accessible_name={props.label}>
    {align === 'start' ? props.avatar : null}
    <column max_width={560} min_width={0}>{props.content}</column>
    {align === 'end' ? props.avatar : null}
  </row>
}

/** Wraps message text and rich content in a themed incoming or outgoing surface. */
export function MessageContent(props: MessageContentProps): JSX.Element {
  const variant = props.variant ?? 'incoming'
  const background = variant === 'outgoing' ? props.theme.accent
    : variant === 'neutral' ? props.theme.surface : props.theme.surfaceRaised
  return <rectangle width="fill" background={background}
    border_color={variant === 'neutral' ? props.theme.border : '#00000000'}
    border_width={variant === 'neutral' ? 1 : 0} radius={props.theme.controlRadius + 3}>
    <column width="fill" min_width={0} gap={6} padding={12} role="group" accessible_name="Message content">
      {props.children}
    </column>
  </rectangle>
}

/** Aligns caller-composed sender and timestamp details above a message. */
export function MessageHeader(props: MessageHeaderProps): JSX.Element {
  return <row width="fill" gap={6} padding_left={3} padding_right={3} align_items="center"
    role="group" accessible_name="Message details">{props.children}</row>
}

/** Aligns caller-composed status, reactions, or actions below a message. */
export function MessageFooter(props: MessageFooterProps): JSX.Element {
  return <row width="fill" gap={6} padding_left={3} padding_right={3} align_items="center"
    role="group" accessible_name="Message status">{props.children}</row>
}

/** Renders the shared themed avatar for a message sender. */
export function MessageAvatar(props: MessageAvatarProps): JSX.Element {
  return <Avatar theme={props.theme} source={props.source} fallback={props.fallback}
    alt={props.alt} size={props.size ?? 32} bordered={true} />
}
