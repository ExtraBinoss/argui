/** @jsxImportSource @argui/react */
import type { ReactElement, ReactNode } from 'react'
import { useState } from 'react'
import { messageScrollOffset, type MessageScrollerProps as SharedMessageScrollerProps } from '../shared/chat-components'

export type ReactMessageScrollerProps = SharedMessageScrollerProps<ReactNode>

/** Provides native vertical scrolling for a growing message history. */
export function ReactMessageScroller(props: ReactMessageScrollerProps): ReactElement {
  const [focused, setFocused] = useState(false)
  const width = props.width ?? 'fill'
  const height = Number.isFinite(props.height) && (props.height ?? 0) > 0 ? props.height! : 320
  return <rectangle nativeKey={props.id} width={width} height={height} clip={true}
    background={props.theme.surface} border_color={focused ? props.theme.accent : props.theme.border}
    border_width={focused ? 2 : 1}
    radius={props.theme.controlRadius}>
    <focusScope nativeKey={`${props.id}-focus`} width="fill" height="fill" focusable={false}
      focus_on_tab_navigation={false} onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}>
      <flickable nativeKey={`${props.id}-viewport`} width="fill" height="fill" scroll_y={true} role="list"
        accessible_name={props.label ?? 'Messages'} orientation="vertical"
        focusable={true} focus_on_tab_navigation={true} can_scroll_into_view={true}
        onScroll={(payload) => {
          const offset = messageScrollOffset(payload)
          if (offset) props.onScroll?.(offset)
        }}>
        <column width="fill" gap={12} padding={12}>{props.children}</column>
      </flickable>
    </focusScope>
  </rectangle>
}
