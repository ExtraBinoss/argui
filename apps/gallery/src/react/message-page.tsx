/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import type { Palette } from '@argui/widgets/react'
import {
  ReactMessage as Message, ReactMessageAvatar as MessageAvatar, ReactMessageContent as MessageContent,
  ReactMessageFooter as MessageFooter, ReactMessageGroup as MessageGroup, ReactMessageHeader as MessageHeader,
} from '../../../../packages/widgets/src/react/message'

/** Demonstrates incoming and outgoing messages with accessible sender metadata. */
export function MessagePage(props: { theme: Palette }): ReactElement {
  return <column width="fill" gap={14}>
    <text width="fill" text="Message rows expose sender and delivery labels to assistive technology. Add buttons, reactions, or attachments to each content surface as needed."
      color={props.theme.muted} font_size={14} />
    <MessageGroup label="Conversation with the Argui team">
      <Message id="chat-message-in" label="Alex, 9:41 AM: The new layout is ready for review." avatar={
        <MessageAvatar theme={props.theme} fallback="AL" alt="Alex Lee" />
      } content={<column gap={5}>
        <MessageHeader>
          <text text="Alex Lee" color={props.theme.foreground} font_size={12} weight={600} />
          <text text="9:41 AM" color={props.theme.muted} font_size={11} />
        </MessageHeader>
        <MessageContent theme={props.theme} variant="incoming">
          <text width="fill" text="The new layout is ready for review. I added the responsive states and keyboard notes."
            color={props.theme.foreground} font_size={14} />
        </MessageContent>
        <MessageFooter>
          <text text="Edited 9:45 AM" color={props.theme.muted} font_size={11} />
        </MessageFooter>
      </column>} />
      <Message id="chat-message-out" label="You, 9:47 AM: Thanks, I will review it now." align="end" avatar={
        <MessageAvatar theme={props.theme} fallback="ME" alt="You" />
      } content={<column gap={5}>
        <MessageHeader>
          <text text="You" color={props.theme.foreground} font_size={12} weight={600} />
          <text text="9:47 AM" color={props.theme.muted} font_size={11} />
        </MessageHeader>
        <MessageContent theme={props.theme} variant="outgoing">
          <text width="fill" text="Thanks, I will review it now." color={props.theme.accentText} font_size={14} />
        </MessageContent>
        <MessageFooter>
          <text text="Delivered" color={props.theme.muted} font_size={11} />
        </MessageFooter>
      </column>} />
    </MessageGroup>
  </column>
}
