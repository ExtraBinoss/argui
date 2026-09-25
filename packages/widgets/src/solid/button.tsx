import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { ButtonProps } from '../shared/types'
import { buttonSize } from '../shared/button-size'
import { useWidgetIcons } from './assets'
import { useButtonGroupJoined } from './button-group'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Composes a keyboard, touch, pointer and accessibility activated native button. */
export function Button(props: ButtonProps): JSX.Element {
  const icons = useWidgetIcons()
  const joined = useButtonGroupJoined()
  const [hovered, setHovered] = createSignal(false)
  const [focused, setFocused] = createSignal(false)
  let pointerFocus = false
  const inactive = () => !!props.disabled || !!props.busy
  const metrics = () => buttonSize(props.size ?? (props.iconOnly ? 'icon' : 'default'), props.theme)
  const fill = (hover: boolean, press: boolean) => props.kind === 'destructive'
    ? press ? props.theme.destructivePressed : hover ? props.theme.destructiveHover : props.theme.destructive
    : props.kind === 'primary'
    ? press ? props.theme.accentPressed : hover ? props.theme.accentHover : props.theme.accent
    : props.kind === 'link' || props.kind === 'quiet' ? '#00000000'
    : props.kind === 'ghost'
      ? press || focused() ? props.theme.surfacePressed
        : hover || props.selected ? props.theme.surfaceHover : '#00000000'
    : press ? props.theme.surfacePressed : hover ? props.theme.surfaceHover
      : props.kind === 'secondary' ? props.theme.surfaceRaised : props.theme.surface
  const color = () => props.disabled ? props.theme.muted
    : props.kind === 'link' ? props.theme.accent
    : props.kind === 'primary' || props.kind === 'destructive' ? props.theme.accentText : props.theme.foreground
  return (
    <focusScope
      key={props.id}
      role={props.role ?? 'button'}
      accessible_name={props.accessibleLabel ?? props.label}
      checked_state={props.role === 'switch' ? props.selected ? 'checked' : 'unchecked' : undefined}
      current={props.current}
      expandable={props.expanded !== undefined ? true : undefined}
      expanded={props.expanded}
      controls={props.controls}
      enabled={!inactive()}
      busy={!!props.busy}
      keyboard_activation="enter_or_space"
      onClick={() => { if (!inactive()) props.onClick() }}
      onFocus={() => { if (!pointerFocus) setFocused(true) }}
      onKey={() => { pointerFocus = false; setFocused(true) }}
      onBlur={() => { pointerFocus = false; setFocused(false) }}
    >
      <touchArea
        enabled={!inactive()}
        mouse_cursor={inactive() ? 'not_allowed' : 'pointer'}
        onPointerEnter={props.icon ? () => { if (!mobile) setHovered(true) } : undefined}
        onPointerLeave={props.icon ? () => setHovered(false) : undefined}
        onPointerDown={() => { pointerFocus = true; setFocused(false) }}
      >
        <rectangle
          width={metrics().iconOnly ? metrics().height : undefined}
          height={metrics().height}
          background={fill(false, false)}
          hover_background={mobile ? undefined : fill(true, false)}
          pressed_background={fill(false, true)}
          border_color={focused() ? props.theme.foreground : !mobile && props.selected ? props.theme.accent : props.theme.border}
          border_width={joined ? focused() ? 1 : 0 : props.kind === 'ghost' || props.kind === 'quiet' || props.kind === 'link' ? 0 : 1}
          radius={joined ? 0 : props.theme.controlRadius}
          opacity={props.disabled ? 0.48 : props.busy ? 0.72 : 1}
        >
          <row width="fill" height="fill" gap={metrics().gap} padding_left={metrics().padding}
            padding_right={metrics().padding} align_items="center" justify_content="center">
            {props.busy ? <rectangle width={16} height={16} rotation_loop_ms={800}>
              {icons.loader ? <svg source={icons.loader} color={color()} width={16} height={16} /> : null}
            </rectangle> : props.icon ? <svg source={(props.selected || hovered()) && props.activeIcon ? props.activeIcon : props.icon}
              color={props.selected || hovered() ? props.theme.accent : color()} width={metrics().icon} height={metrics().icon}
              rotation={props.iconRotation} /> : null}
            {!metrics().iconOnly && !props.iconOnly ? <text text={props.label} color={color()} font_size={metrics().font} /> : null}
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
  )
}
