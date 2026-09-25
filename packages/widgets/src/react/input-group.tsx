/** @jsxImportSource @argui/react */
import { createContext, createElement, useContext, useRef, useState, type ReactElement, type ReactNode } from 'react'
import { InputEditController } from '../shared/input-edit'
import { selectionTint } from '../shared/theme'
import type { Palette } from '../shared/theme'

/** Horizontal or vertical arrangement for an input group. */
export type InputGroupOrientation = 'horizontal' | 'vertical'
/** Placement label for an input-group adornment. */
export type InputGroupAlign = 'inline-start' | 'inline-end' | 'block-start' | 'block-end'

interface InputGroupContextValue {
  /** Reports focus from a group control so the surrounding border can respond. */
  setFocused(focused: boolean): void
}

const InputGroupContext = createContext<InputGroupContextValue | undefined>(undefined)

/** Props for a bordered group that combines an editor with labels or actions. */
export interface InputGroupProps {
  /** Colors and sizing for the group and its child controls. */
  theme: Palette
  /** Optional accessible group name. */
  label?: string
  /** Controls and addons in visual order; place start addons before the editor. */
  children: ReactNode
  /** Layout direction. Use vertical when placing block-aligned addons. */
  orientation?: InputGroupOrientation
  /** Whether all child controls should appear disabled. */
  disabled?: boolean
  /** Whether the group is invalid. */
  invalid?: boolean
  /** Gap between the controls and addons. */
  gap?: number
}

/** Renders a themed input-group frame and highlights it while one of its editors is focused. */
export function ReactInputGroup(props: InputGroupProps): ReactElement {
  const [focused, setFocused] = useState(false)
  const orientation = props.orientation ?? 'horizontal'
  return createElement(InputGroupContext.Provider, { value: { setFocused } },
    <focusScope width="fill" role="group" accessible_name={props.label} invalid={!!props.invalid}
      accessible_disabled={!!props.disabled}>
      <rectangle width="fill" min_height={props.theme.inputHeight}
        background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
        border_color={props.invalid ? props.theme.destructive : focused ? props.theme.accent : props.theme.border}
        border_width={1} radius={props.theme.controlRadius}>
        {orientation === 'vertical'
          ? <column width="fill" min_width={0} gap={props.gap ?? 6} padding={props.theme.controlPadding}>{props.children}</column>
          : <row width="fill" min_width={0} gap={props.gap ?? 6} padding_left={props.theme.controlPadding}
            padding_right={props.theme.controlPadding} align_items="center">{props.children}</row>}
      </rectangle>
    </focusScope>
  ) as ReactElement
}

/** Props for a labeled adornment adjacent to an input-group editor. */
export interface InputGroupAddonProps {
  /** Colors and sizing for the adornment. */
  theme: Palette
  /** Addon placement; siblings remain in caller-provided order. */
  align?: InputGroupAlign
  /** Visible text, icon, or action content. */
  children: ReactNode
  /** Optional action when the addon itself is clicked. */
  onClick?: () => void
}

/** Renders an addon without taking focus away from its nested controls. */
export function ReactInputGroupAddon(props: InputGroupAddonProps): ReactElement {
  const block = props.align === 'block-start' || props.align === 'block-end'
  return <row width={block ? 'fill' : undefined} shrink={block ? undefined : 0} gap={8}
    align_items="center" justify_content={block ? 'start' : undefined} onClick={props.onClick}>
    {props.children}
  </row>
}

/** Props for muted text or custom explanatory content beside an editor. */
export interface InputGroupTextProps {
  /** Palette used for subdued text. */
  theme: Palette
  /** Text shown beside the editor when children are not supplied. */
  text?: string
  /** Optional custom content such as a small icon. */
  children?: ReactNode
}

/** Renders muted inline text inside an input group. */
export function ReactInputGroupText(props: InputGroupTextProps): ReactElement {
  return <row shrink={0} align_items="center">
    {props.children ?? <text text={props.text ?? ''} color={props.theme.muted} font_size={12} />}
  </row>
}

/** Props for a compact action button placed inside an input group. */
export interface InputGroupButtonProps {
  /** Stable key used by the native focus system. */
  id: string
  /** Palette used for the action surface. */
  theme: Palette
  /** Accessible name for the action. */
  label: string
  /** Action called by pointer, touch, Enter, or Space. */
  onClick: () => void
  /** Optional visible label or icon. */
  children?: ReactNode
  /** Disables pointer and keyboard activation. */
  disabled?: boolean
  /** Background style for the compact action. */
  variant?: 'default' | 'outline' | 'ghost'
  /** Compact text or icon-only dimensions. */
  size?: 'sm' | 'icon'
}

/** Renders a keyboard- and touch-activated compact button for a group addon. */
export function ReactInputGroupButton(props: InputGroupButtonProps): ReactElement {
  const [pressed, setPressed] = useState(false)
  const [hovered, setHovered] = useState(false)
  const variant = props.variant ?? 'ghost'
  const background = variant === 'default' ? props.theme.accent
    : variant === 'outline' ? pressed ? props.theme.surfacePressed : props.theme.surface
    : pressed || hovered ? props.theme.surfaceHover : '#00000000'
  const color = variant === 'default' ? props.theme.accentText : props.theme.foreground
  return <focusScope nativeKey={props.id} role="button" accessible_name={props.label} enabled={!props.disabled}
    keyboard_activation="enter_or_space" onClick={() => { if (!props.disabled) props.onClick() }}>
    <touchArea enabled={!props.disabled} mouse_cursor={props.disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => setHovered(true)} onPointerLeave={() => { setHovered(false); setPressed(false) }}
      onPointerDown={() => setPressed(true)} onPointerUp={() => setPressed(false)} onPointerCancel={() => setPressed(false)}>
      <rectangle width={props.size === 'icon' ? 30 : undefined} height={30} background={background}
        border_color={variant === 'outline' ? props.theme.border : '#00000000'}
        border_width={variant === 'outline' ? 1 : 0} radius={props.theme.controlRadius}
        opacity={props.disabled ? 0.48 : 1}>
        <row width="fill" height="fill" padding_left={props.size === 'icon' ? 0 : 9}
          padding_right={props.size === 'icon' ? 0 : 9} align_items="center" justify_content="center">
          {props.children ?? <text text={props.label} color={color} font_size={12} />}
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}

/** Props shared by single-line and multiline input-group controls. */
export interface InputGroupControlProps {
  /** Stable key used to retain the native editor. */
  id: string
  /** Palette used for text, caret, selection, and disabled state. */
  theme: Palette
  /** Accessible name for the editor. */
  label: string
  /** Controlled value. Omit to use `defaultValue` and internal state. */
  value?: string
  /** Initial text for an uncontrolled editor. */
  defaultValue?: string
  /** Called with the complete edited string. */
  onChange?: (value: string) => void
  /** Placeholder shown while empty. */
  placeholder?: string
  /** Supporting description announced by the native editor. */
  description?: string
  /** Optional visible-label key referenced by `labelled_by`. */
  labelledBy?: string
  /** Optional helper-text key referenced by `described_by`. */
  describedBy?: string
  /** Whether the editor is disabled. */
  disabled?: boolean
  /** Whether the editor is read-only. */
  readOnly?: boolean
  /** Whether to expose the editor as required. */
  required?: boolean
  /** Whether the editor is invalid. */
  invalid?: boolean
  /** Whether the host should offer search-entry semantics. */
  search?: boolean
  /** Optional host-level ASCII digit limit. */
  maxDigits?: number
  /** Text-selection color override. */
  selectionColor?: string
}

/** Props for a single-line native editor inside an input group. */
export type InputGroupInputProps = InputGroupControlProps

/** Renders an accessible native text editor that expands between group addons. */
export function ReactInputGroupInput(props: InputGroupInputProps): ReactElement {
  return useInputGroupControl(props, false)
}

/** Props for a multiline editor placed in a vertically arranged input group. */
export interface InputGroupTextareaProps extends InputGroupControlProps {
  /** Editor height in logical pixels. Defaults to 112. */
  height?: number
}

/** Renders a native multiline editor inside a vertically arranged input group. */
export function ReactInputGroupTextarea(props: InputGroupTextareaProps): ReactElement {
  return useInputGroupControl(props, true)
}

function useInputGroupControl(props: InputGroupControlProps & { height?: number }, multiline: boolean): ReactElement {
  const [uncontrolled, setUncontrolled] = useState(props.value ?? props.defaultValue ?? '')
  const group = useContext(InputGroupContext)
  const value = props.value !== undefined ? props.value : uncontrolled
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(value)
  const height = props.height ?? (multiline ? 112 : props.theme.inputHeight)
  return <container width="fill" min_width={0} grow={1} height={height}>
    <textInput nativeKey={props.id} width="fill" height="fill" clip={true} role={multiline ? 'text_area' : 'text_input'}
      multiline={multiline} value={value} placeholder={props.placeholder ?? ''}
      label={props.label} description={props.description} labelled_by={props.labelledBy}
      described_by={props.describedBy}
      enabled={!props.disabled} read_only={!!props.readOnly} required={!!props.required} invalid={!!props.invalid}
      search={!!props.search} max_digits={props.maxDigits} background="#00000000"
      text_color={props.disabled ? props.theme.muted : props.theme.foreground}
      placeholder_color={props.theme.muted} caret_color={props.theme.accent}
      selection_color={props.selectionColor ?? selectionTint(props.theme.accent)}
      onFocus={() => group?.setFocused(true)} onBlur={() => group?.setFocused(false)}
      onEdit={(payload) => edits.current!.apply(payload, value, (next) => {
        if (props.value === undefined) setUncontrolled(next)
        props.onChange?.(next)
      })} />
  </container>
}
