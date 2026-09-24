import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { ButtonProps, PopoverProps as SharedPopoverProps, SelectProps } from '../shared/types'
import { useWidgetIcons } from './assets'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Composes a keyboard, touch, pointer and accessibility activated native button. */
export function Button(props: ButtonProps): JSX.Element {
  const icons = useWidgetIcons()
  const [hovered, setHovered] = createLocalSignal(false)
  const [pressed, setPressed] = createLocalSignal(false)
  const [focused, setFocused] = createLocalSignal(false)
  let pointerFocus = false
  const inactive = () => !!props.disabled || !!props.busy
  const fill = () => props.kind === 'destructive'
    ? pressed() ? props.theme.destructivePressed : hovered() ? props.theme.destructiveHover : props.theme.destructive
    : props.kind === 'primary'
    ? pressed() ? props.theme.accentPressed : hovered() ? props.theme.accentHover : props.theme.accent
    : props.kind === 'ghost'
      ? pressed() || focused() ? props.theme.surfacePressed
        : hovered() || props.selected ? props.theme.surfaceHover : '#00000000'
    : pressed() ? props.theme.surfacePressed : hovered() ? props.theme.surfaceHover
      : props.kind === 'secondary' ? props.theme.surfaceRaised : props.theme.surface
  const color = () => props.disabled ? props.theme.muted
    : props.kind === 'primary' || props.kind === 'destructive' ? props.theme.accentText : props.theme.foreground
  return (
    <focusScope
      key={props.id}
      role={props.role ?? 'button'}
      accessible_name={props.label}
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
        onPointerEnter={() => { if (!mobile) setHovered(true) }}
        onPointerLeave={() => { setHovered(false); setPressed(false) }}
        onPointerDown={() => { pointerFocus = true; setPressed(true); setFocused(false) }}
        onPointerUp={() => setPressed(false)}
        onPointerCancel={() => setPressed(false)}
      >
        <rectangle
          background={fill()}
          border_color={focused() ? props.theme.foreground : !mobile && props.selected ? props.theme.accent : props.theme.border}
          border_width={props.kind === 'ghost' ? 0 : 1}
          radius={9}
          opacity={props.disabled ? 0.48 : props.busy ? 0.72 : 1}
        >
          <row gap={8} padding={10} align_items="center">
            {props.busy ? <rectangle width={16} height={16} rotation_loop_ms={800}>
              {icons.loader ? <svg source={icons.loader} color={color()} width={16} height={16} /> : null}
            </rectangle> : props.icon ? <svg source={(props.selected || hovered()) && props.activeIcon ? props.activeIcon : props.icon}
              color={props.selected || hovered() ? props.theme.accent : color()} width={18} height={18} /> : null}
            {!props.iconOnly || props.busy ? <text text={props.label} color={color()} font_size={14} /> : null}
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
  )
}

/** Composes a controlled native combobox and anchored option popup. */
export function Select(props: SelectProps): JSX.Element {
  const icons = useWidgetIcons()
  const [expanded, setExpanded] = createLocalSignal(false)
  const [activeIndex, setActiveIndex] = createLocalSignal(0)
  const [focused, setFocused] = createLocalSignal(false)
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
    const key = typeof payload === 'object' && payload && 'key' in payload
      ? String(payload.key) : String(payload)
    if (key === 'Escape') setExpanded(false)
    else if (key === 'ArrowDown' || key === 'ArrowUp') {
      if (props.options.length === 0) return
      const step = key === 'ArrowDown' ? 1 : -1
      const index = (activeIndex() + step + props.options.length) % props.options.length
      setActiveIndex(index)
      setExpanded(true)
    } else if (key === 'Enter' || key === ' ') {
      if (expanded()) {
        const option = props.options[activeIndex()]
        if (option !== undefined) choose(option)
      } else open()
    }
  }
  return (
    <column gap={8}>
      <text text={props.label} color={props.theme.muted} font_size={12} />
      <focusScope
        key={props.id}
        role="combo_box"
        accessible_name={`${props.label}: ${props.value}`}
        accessible_value={props.value}
        controls={`${props.id}-popup`}
        active_descendant={expanded() ? `${props.id}-option-${activeIndex()}` : undefined}
        expandable={true}
        expanded={expanded()}
        keyboard_activation="none"
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        onClick={() => expanded() ? setExpanded(false) : open()}
        onKey={onKey}
      >
        <rectangle width={240} height={40} background={props.theme.surface}
          border_color={focused() ? props.theme.accent : props.theme.border}
          border_width={focused() ? 2 : 1} radius={8}>
          <row width="fill" height="fill" padding={10} align_items="center">
            <text text={props.value} color={props.theme.foreground} font_size={14} />
            <container grow={1} />
            {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.muted} width={16} height={16} rotation={expanded() ? 180 : 0} /> : null}
          </row>
        </rectangle>
      </focusScope>
      {expanded() ? (
        <popupWindow key={`${props.id}-popup`} anchor={props.id} placement="bottom_start" width={240}
          window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap" initial_focus="first"
          restore_focus={true} onDismiss={() => setExpanded(false)}>
          <focusScope role="list_box" accessible_name={props.label} focus_on_tab_navigation={false}>
            <rectangle width={240} background={props.theme.surface}
              border_color={props.theme.border} border_width={1} radius={8}
              shadow_blur={14} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
              <column width="fill" gap={2} padding={4}>
                {props.options.map((option, index) => (
                  <focusScope key={`${props.id}-option-${index}`} role="option"
                    accessible_name={option} selected={props.value === option}
                    keyboard_activation="enter_or_space" onClick={() => choose(option)}>
                    <touchArea mouse_cursor="pointer" onPointerEnter={() => setActiveIndex(index)}>
                      <rectangle width="fill" height={34}
                        background={index === activeIndex() || option === props.value ? props.theme.surfaceRaised : props.theme.surface}
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

export type PopoverProps = SharedPopoverProps<JSX.Element>

/** Presents arbitrary content in a themed, anchored native popover. */
export function Popover(props: PopoverProps): JSX.Element {
  const [localExpanded, setLocalExpanded] = createLocalSignal(false)
  const expanded = () => props.open ?? localExpanded()
  const setExpanded = (open: boolean) => {
    if (props.onOpenChange) props.onOpenChange(open)
    else setLocalExpanded(open)
  }
  return <column>
    <Button id={props.id} label={props.label} theme={props.theme} kind="secondary"
      expanded={expanded()} controls={`${props.id}-popup`} onClick={() => setExpanded(!expanded())} />
    {expanded() ? <popupWindow key={`${props.id}-popup`} anchor={props.id} placement="bottom_start" width={260}
      window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap"
      initial_focus="first" restore_focus={true} onDismiss={() => setExpanded(false)}>
      <rectangle width={260} background={props.opaque ? props.theme.surface : props.theme.overlaySurface}
        backdrop_filter={props.opaque || props.blur === false ? undefined : 'blur(10px)'}
        border_color={props.theme.border} border_width={1} radius={10}
        shadow_blur={14} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={12} padding={16}>
          {props.children}
          <Button id={`${props.id}-close`} label="Close" theme={props.theme} kind="outline"
            onClick={() => setExpanded(false)} />
        </column>
      </rectangle>
    </popupWindow> : null}
  </column>
}

/** Creates a local Solid signal while keeping control state outside the native host. */
function createLocalSignal<T>(initial: T): [() => T, (value: T) => void] {
  const [get, set] = createSignal(initial)
  return [get, (value) => { set(() => value) }]
}
