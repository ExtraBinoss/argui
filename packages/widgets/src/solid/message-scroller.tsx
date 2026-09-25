import type { JSX } from '@argui/solid/jsx-runtime'
import { createSignal } from '@argui/solid'
import { messageScrollOffset, type MessageScrollerProps as SharedMessageScrollerProps } from '../shared/chat-components'

export type MessageScrollerProps = SharedMessageScrollerProps<JSX.Element>

/** Provides native vertical scrolling for a growing message history. */
export function MessageScroller(props: MessageScrollerProps): JSX.Element {
  const [focused, setFocused] = createSignal(false)
  const width = props.width ?? 'fill'
  const height = Number.isFinite(props.height) && (props.height ?? 0) > 0 ? props.height! : 320
  return <rectangle key={props.id} width={width} height={height} clip={true}
    background={props.theme.surface} border_color={focused() ? props.theme.accent : props.theme.border}
    border_width={focused() ? 2 : 1}
    radius={props.theme.controlRadius}>
    <focusScope key={`${props.id}-focus`} width="fill" height="fill" focusable={false}
      focus_on_tab_navigation={false} onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}>
      <flickable key={`${props.id}-viewport`} width="fill" height="fill" scroll_y={true} role="list"
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
