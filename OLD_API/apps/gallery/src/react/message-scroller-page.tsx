/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { MessageScrollOffset } from '../../../../packages/widgets/src/shared/chat-components'
import type { Palette } from '@argui/widgets/react'
import { ReactMessageScroller as MessageScroller } from '../../../../packages/widgets/src/react/message-scroller'
import {
  ReactMessage as Message, ReactMessageAvatar as MessageAvatar,
  ReactMessageContent as MessageContent, ReactMessageHeader as MessageHeader,
} from '../../../../packages/widgets/src/react/message'

const messages = [
  ['Maya', 'The design review is at 2 PM.'],
  ['You', 'I will bring the keyboard navigation notes.'],
  ['Omar', 'I tested the new message layout on a narrow window.'],
  ['Maya', 'Great. I will add the final screenshots after the review.'],
  ['You', 'The attachments are ready in the project thread.'],
  ['Omar', 'I can check the dark theme states as well.'],
  ['Maya', 'Thanks, please include the focus states in that pass.'],
  ['You', 'Done. I will post a summary when the build finishes.'],
]

/** Demonstrates native scrolling and reports its current vertical offset. */
export function MessageScrollerPage(props: { theme: Palette }): ReactElement {
  const [offset, setOffset] = useState<MessageScrollOffset>({ x: 0, y: 0 })
  return <column width="fill" gap={12}>
    <text width="fill" text="Scroll the conversation with the wheel, touch, or focused keyboard navigation. The host reports the scroll offset; it does not expose a programmatic scroll-to-latest command."
      color={props.theme.muted} font_size={14} />
    <MessageScroller id="chat-scroller-demo" theme={props.theme} label="Scrollable team conversation"
      height={340} onScroll={setOffset}>
      {messages.map(([sender, body], index) => <Message key={`scroll-message-${index}`}
        id={`scroll-message-${index}`} label={`${sender}: ${body}`} align={sender === 'You' ? 'end' : 'start'}
        avatar={<MessageAvatar theme={props.theme} fallback={sender.slice(0, 2).toUpperCase()} alt={sender} />}
        content={<column gap={4}>
          <MessageHeader>
            <text text={sender} color={props.theme.foreground} font_size={12} weight={600} />
            <text text={`${9 + Math.floor(index / 4)}:${String(10 + index * 6).padStart(2, '0')} AM`}
              color={props.theme.muted} font_size={11} />
          </MessageHeader>
          <MessageContent theme={props.theme} variant={sender === 'You' ? 'outgoing' : 'incoming'}>
            <text width="fill" text={body}
              color={sender === 'You' ? props.theme.accentText : props.theme.foreground} font_size={13} />
          </MessageContent>
        </column>} />)}
    </MessageScroller>
    <text width="fill" text={`Scroll position: ${Math.round(offset.x)}, ${Math.round(offset.y)} px`}
      color={props.theme.foreground} font_size={12} role="status" live="polite" />
  </column>
}
