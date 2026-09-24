/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import type { ButtonProps } from '../shared/types'
import { useWidgetIcons } from './assets'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

export type ReactButtonProps = ButtonProps

/** Composes the same native button semantics as the Solid gallery. */
export function ReactButton(props: ReactButtonProps): ReactElement {
  const icons = useWidgetIcons()
  const [hovered, setHovered] = useState(false)
  const [pressed, setPressed] = useState(false)
  const [focused, setFocused] = useState(false)
  const pointerFocus = useRef(false)
  const inactive = !!props.disabled || !!props.busy
  const fill = props.kind === 'destructive'
    ? pressed ? props.theme.destructivePressed : hovered ? props.theme.destructiveHover : props.theme.destructive
    : props.kind === 'primary'
    ? pressed ? props.theme.accentPressed : hovered ? props.theme.accentHover : props.theme.accent
    : props.kind === 'ghost'
      ? pressed || focused ? props.theme.surfacePressed
        : hovered || props.selected ? props.theme.surfaceHover : '#00000000'
    : pressed ? props.theme.surfacePressed : hovered ? props.theme.surfaceHover
      : props.kind === 'secondary' ? props.theme.surfaceRaised : props.theme.surface
  const color = props.disabled ? props.theme.muted
    : props.kind === 'primary' || props.kind === 'destructive' ? props.theme.accentText : props.theme.foreground
  return (
    <focusScope
      nativeKey={props.id}
      role={props.role ?? 'button'}
      accessible_name={props.label}
      checked_state={props.role === 'switch' ? props.selected ? 'checked' : 'unchecked' : undefined}
      current={props.current}
      expandable={props.expanded !== undefined ? true : undefined}
      expanded={props.expanded}
      controls={props.controls}
      enabled={!inactive}
      busy={!!props.busy}
      keyboard_activation="enter_or_space"
      onClick={() => { if (!inactive) props.onClick() }}
      onFocus={() => { if (!pointerFocus.current) setFocused(true) }}
      onKey={() => { pointerFocus.current = false; setFocused(true) }}
      onBlur={() => { pointerFocus.current = false; setFocused(false) }}
    >
      <touchArea enabled={!inactive} mouse_cursor={inactive ? 'not_allowed' : 'pointer'}
        onPointerEnter={() => { if (!mobile) setHovered(true) }}
        onPointerLeave={() => { setHovered(false); setPressed(false) }}
        onPointerDown={() => { pointerFocus.current = true; setPressed(true); setFocused(false) }} onPointerUp={() => setPressed(false)}
        onPointerCancel={() => setPressed(false)}>
        <rectangle background={fill} border_color={focused ? props.theme.foreground : !mobile && props.selected ? props.theme.accent : props.theme.border}
          border_width={props.kind === 'ghost' ? 0 : 1} radius={9}
          opacity={props.disabled ? 0.48 : props.busy ? 0.72 : 1}>
          <row gap={8} padding={10} align_items="center">
            {props.busy ? <rectangle width={16} height={16} rotation_loop_ms={800}>
              {icons.loader ? <svg source={icons.loader} color={color} width={16} height={16} /> : null}
            </rectangle> : props.icon ? <svg source={(props.selected || hovered) && props.activeIcon ? props.activeIcon : props.icon}
              color={props.selected || hovered ? props.theme.accent : color} width={18} height={18} /> : null}
            {!props.iconOnly || props.busy ? <text text={props.label} color={color} font_size={14} /> : null}
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
  )
}
