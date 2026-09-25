/** @jsxImportSource @argui/react */
import { useState, type ReactElement, type ReactNode } from 'react'
import type { SurfaceScrollAreaProps } from '../shared/surface-c'

export type ReactScrollAreaProps = SurfaceScrollAreaProps<ReactNode>

/** Renders a fixed-size viewport that scrolls with the same native input handling. */
export function ReactScrollArea(props: ReactScrollAreaProps): ReactElement {
  const [focused, setFocused] = useState(false)
  const orientation = props.orientation ?? 'vertical'
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 320
  const height = Number.isFinite(props.height) && (props.height ?? 0) > 0 ? props.height! : 240
  const padding = Number.isFinite(props.padding) && (props.padding ?? -1) >= 0 ? props.padding! : 12
  if (orientation === 'vertical') {
    return <rectangle width={width} height={height} background={props.theme.surface}
      border_color={focused ? props.theme.accent : props.theme.border}
      border_width={focused ? 2 : 1} radius={props.theme.controlRadius} clip={true}>
      <focusScope nativeKey={`${props.id}-focus`} width="fill" height="fill" focusable={false}
        focus_on_tab_navigation={false} onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}>
        <column nativeKey={props.id} width="fill" height="fill" padding={padding} gap={8}
          scroll_y={true} scrollbar_thumb={props.theme.muted} role="group" accessible_name={props.label}
          orientation="vertical" focusable={true} focus_on_tab_navigation={true}>
          {props.children}
        </column>
      </focusScope>
    </rectangle>
  }
  const horizontal = orientation === 'horizontal' || orientation === 'both'
  const vertical = orientation === 'both'
  return <rectangle width={width} height={height} background={props.theme.surface}
    border_color={focused ? props.theme.accent : props.theme.border}
    border_width={focused ? 2 : 1} radius={props.theme.controlRadius} clip={true}>
    <focusScope nativeKey={`${props.id}-focus`} width="fill" height="fill" focusable={false}
      focus_on_tab_navigation={false} onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}>
      <flickable nativeKey={props.id} width="fill" height="fill" scroll_x={horizontal} scroll_y={vertical}
        role="group" accessible_name={props.label} orientation={vertical ? undefined : 'horizontal'}
        focusable={true} focus_on_tab_navigation={true}>
        <column padding={padding}>{props.children}</column>
      </flickable>
    </focusScope>
  </rectangle>
}
