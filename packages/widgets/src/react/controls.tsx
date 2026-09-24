/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement, type ReactNode } from 'react'
import type { ButtonProps, PopoverProps as SharedPopoverProps, SelectProps } from '../shared/types'
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

export type ReactSelectProps = SelectProps

/** Composes a controlled native combobox with an anchored option popup. */
export function ReactSelect(props: ReactSelectProps): ReactElement {
  const icons = useWidgetIcons()
  const [expanded, setExpanded] = useState(false)
  const [activeIndex, setActiveIndex] = useState(0)
  const [focused, setFocused] = useState(false)
  const open = () => {
    setActiveIndex(Math.max(0, props.options.indexOf(props.value)))
    setExpanded(true)
  }
  const choose = (value: string) => {
    props.onChange(value)
    setExpanded(false)
  }
  const onKey = (payload: unknown) => {
    if (typeof payload === 'object' && payload && 'state' in payload && payload.state !== 'pressed') return
    const key = typeof payload === 'object' && payload && 'key' in payload ? String(payload.key) : String(payload)
    if (key === 'Escape') setExpanded(false)
    else if (key === 'ArrowDown' || key === 'ArrowUp') {
      if (props.options.length === 0) return
      const step = key === 'ArrowDown' ? 1 : -1
      setActiveIndex((index) => (index + step + props.options.length) % props.options.length)
      setExpanded(true)
    } else if (key === 'Enter' || key === ' ') {
      if (expanded) {
        const option = props.options[activeIndex]
        if (option !== undefined) choose(option)
      } else open()
    }
  }
  return (
    <column gap={8}>
      <text text={props.label} color={props.theme.muted} font_size={12} />
      <focusScope nativeKey={props.id} role="combo_box"
        accessible_name={`${props.label}: ${props.value}`} accessible_value={props.value}
        controls={`${props.id}-popup`}
        active_descendant={expanded ? `${props.id}-option-${activeIndex}` : undefined}
        expandable={true} expanded={expanded} keyboard_activation="none"
        onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
        onClick={() => expanded ? setExpanded(false) : open()} onKey={onKey}>
        <rectangle width={240} height={40} background={props.theme.surface}
          border_color={focused ? props.theme.accent : props.theme.border}
          border_width={focused ? 2 : 1} radius={8}>
          <row width="fill" height="fill" padding={10} align_items="center">
            <text text={props.value} color={props.theme.foreground} font_size={14} />
            <container grow={1} />
            {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.muted} width={16} height={16} rotation={expanded ? 180 : 0} /> : null}
          </row>
        </rectangle>
      </focusScope>
      {expanded ? (
        <popupWindow nativeKey={`${props.id}-popup`} anchor={props.id} placement="bottom_start"
          width={240} window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap" initial_focus="first" restore_focus={true}
          onDismiss={() => setExpanded(false)}>
          <focusScope role="list_box" accessible_name={props.label} focus_on_tab_navigation={false}>
            <rectangle width={240} background={props.theme.surface}
              border_color={props.theme.border} border_width={1} radius={8}
              shadow_blur={14} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
              <column width="fill" gap={2} padding={4}>
                {props.options.map((option, index) => (
                  <focusScope key={option} nativeKey={`${props.id}-option-${index}`} role="option"
                    accessible_name={option} selected={props.value === option}
                    keyboard_activation="enter_or_space" onClick={() => choose(option)}>
                    <touchArea mouse_cursor="pointer" onPointerEnter={() => setActiveIndex(index)}>
                      <rectangle width="fill" height={34}
                        background={index === activeIndex || option === props.value ? props.theme.surfaceRaised : props.theme.surface}
                        radius={5}>
                        <row width="fill" height="fill" padding={10} align_items="center">
                          <text text={option} color={props.theme.foreground} font_size={14} />
                        </row>
                      </rectangle>
                    </touchArea>
                  </focusScope>
                ))}
              </column>
            </rectangle>
          </focusScope>
        </popupWindow>
      ) : null}
    </column>
  )
}

export type ReactPopoverProps = SharedPopoverProps<ReactNode>

/** Presents arbitrary content in the same anchored native popover as Solid. */
export function ReactPopover(props: ReactPopoverProps): ReactElement {
  const [localExpanded, setLocalExpanded] = useState(false)
  const expanded = props.open ?? localExpanded
  const setExpanded = (open: boolean) => {
    if (props.onOpenChange) props.onOpenChange(open)
    else setLocalExpanded(open)
  }
  return <column>
    <ReactButton id={props.id} label={props.label} theme={props.theme} kind="secondary"
      expanded={expanded} controls={`${props.id}-popup`} onClick={() => setExpanded(!expanded)} />
    {expanded ? <popupWindow nativeKey={`${props.id}-popup`} anchor={props.id} placement="bottom_start" width={260}
      window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap"
      initial_focus="first" restore_focus={true} onDismiss={() => setExpanded(false)}>
      <rectangle width={260} background={props.opaque ? props.theme.surface : props.theme.overlaySurface}
        backdrop_filter={props.opaque || props.blur === false ? undefined : 'blur(10px)'}
        border_color={props.theme.border} border_width={1} radius={10}
        shadow_blur={14} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={12} padding={16}>
          {props.children}
          <ReactButton id={`${props.id}-close`} label="Close" theme={props.theme} kind="outline"
            onClick={() => setExpanded(false)} />
        </column>
      </rectangle>
    </popupWindow> : null}
  </column>
}
