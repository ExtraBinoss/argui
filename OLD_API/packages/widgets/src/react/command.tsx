/** @jsxImportSource @argui/react */
import { createContext, createElement, useCallback, useContext, useEffect, useRef, useState,
  type ReactElement, type ReactNode } from 'react'
import { foundationIKey } from '../shared/foundation-i'
import { foundationJFilter, foundationJNext, type CommandEntry } from '../shared/foundation-j'
import { InputEditController } from '../shared/input-edit'
import { selectionTint, type Palette } from '../shared/theme'
import { ReactDialog } from './dialog'

type RegisteredCommandEntry = CommandEntry & { onSelect?: () => void }
interface CommandContextValue {
  id(): string
  theme(): Palette
  label(): string
  disabled(): boolean
  search(): string
  setSearch(value: string): void
  matches(): readonly RegisteredCommandEntry[]
  active(): string | undefined
  setActive(value: string): void
  register(entry: RegisteredCommandEntry): () => void
  select(value: string): void
  selected(value: string): boolean
  isVisible(value: string): boolean
  groupHasResults(group?: string): boolean
  emptyLabel(): string
  onKey(payload: unknown): void
}

const CommandContext = createContext<CommandContextValue | null>(null)
const CommandGroupContext = createContext<string | undefined>(undefined)

/** Props for a searchable, keyboard-operated command list. */
export interface CommandProps {
  /** Stable prefix used by the search editor, list, and item targets. */ id: string
  /** Accessible label for the command menu. */ label: string
  /** Palette used for the command surface and rows. */ theme: Palette
  /** Search input, list, groups, and command items. */ children: ReactNode
  /** Controlled search text; pair with `onSearchChange`. */ search?: string
  /** Initial search text used only when `search` is omitted. */ defaultSearch?: string
  /** Called when the search text changes. */ onSearchChange?: (search: string) => void
  /** Controlled last-selected command value. */ value?: string
  /** Initial selected command value used only when `value` is omitted. */ defaultValue?: string
  /** Called when the selected command value changes. */ onValueChange?: (value: string) => void
  /** Called when a command item is selected by pointer or keyboard. */ onSelect?: (value: string) => void
  /** Text shown when no item matches the current query. */ emptyLabel?: string
  /** Disables all search and command actions. */ disabled?: boolean
}

/** Props for a command palette rendered inside the native modal dialog. */
export interface CommandDialogProps {
  /** Stable native key for the dialog. */ id: string
  /** Modal heading announced to assistive technology. */ title: string
  /** Supporting copy announced with the title. */ description?: string
  /** Whether the command palette is visible. */ open: boolean
  /** Called when Escape, the close action, or outside dismissal closes it. */ onOpenChange(open: boolean): void
  /** Palette used by the dialog and command list. */ theme: Palette
  /** A `Command` root and its input, list, and items. */ children: ReactNode
  /** Dialog width in logical pixels. */ width?: number
  /** Close-button label; pass false to omit the footer action. Defaults to Close. */ closeLabel?: string | false
}

/** Places a composed command palette in Argui's accessible modal dialog. */
export function ReactCommandDialog(props: CommandDialogProps): ReactElement | null {
  return <ReactDialog id={props.id} title={props.title} description={props.description} open={props.open}
    onOpenChange={props.onOpenChange} theme={props.theme} width={props.width}
    closeLabel={props.closeLabel ?? 'Close'}>
    {props.children}
  </ReactDialog>
}

/** Provides search, selection, filtering, and roving active-descendant behavior. */
export function ReactCommand(props: CommandProps): ReactElement {
  const [uncontrolledSearch, setUncontrolledSearch] = useState(props.defaultSearch ?? '')
  const [uncontrolledValue, setUncontrolledValue] = useState(props.defaultValue ?? '')
  const entriesRef = useRef<RegisteredCommandEntry[]>([])
  const [revision, setRevision] = useState(0)
  const [activeValue, setActiveValue] = useState<string | undefined>(undefined)
  const search = () => props.search ?? uncontrolledSearch
  const selectedValue = () => props.value ?? uncontrolledValue
  const matches = () => {
    revision
    return foundationJFilter(entriesRef.current, search())
  }
  const active = () => {
    const visible = matches()
    return visible.find((entry) => entry.value === activeValue && !entry.disabled)?.value
      ?? visible.find((entry) => !entry.disabled)?.value
  }
  const setActive = (value: string) => {
    if (matches().some((entry) => entry.value === value && !entry.disabled)) setActiveValue(value)
  }
  const setSearch = (next: string) => {
    if (props.search === undefined) setUncontrolledSearch(next)
    props.onSearchChange?.(next)
    const nextMatch = foundationJFilter(entriesRef.current, next).find((entry) => !entry.disabled)
    setActiveValue(nextMatch?.value)
  }
  const register = useCallback((entry: RegisteredCommandEntry) => {
    const current = entriesRef.current
    entriesRef.current = current.some((item) => item.value === entry.value)
      ? current.map((item) => item.value === entry.value ? entry : item)
      : [...current, entry]
    setRevision((count) => count + 1)
    return () => {
      entriesRef.current = entriesRef.current.filter((item) => item.value !== entry.value)
      setRevision((count) => count + 1)
    }
  }, [])
  const select = (value: string) => {
    const entry = matches().find((item) => item.value === value && !item.disabled)
    if (!entry || props.disabled) return
    if (props.value === undefined) setUncontrolledValue(value)
    props.onValueChange?.(value)
    props.onSelect?.(value)
    entry.onSelect?.()
  }
  const onKey = (payload: unknown) => {
    const key = foundationIKey(payload)
    if (!key || props.disabled) return
    const visible = matches()
    if (key === 'ArrowDown' || key === 'ArrowUp') {
      const next = foundationJNext(visible, active(), key === 'ArrowDown' ? 1 : -1)
      if (next) setActive(next)
    } else if (key === 'Home' || key === 'End') {
      const enabled = visible.filter((entry) => !entry.disabled)
      const edge = key === 'Home' ? enabled[0] : enabled.at(-1)
      if (edge) setActive(edge.value)
    } else if (key === 'Enter') {
      const current = active()
      if (current) select(current)
    } else if (key === 'Escape' && search()) setSearch('')
  }
  const context: CommandContextValue = {
    id: () => props.id, theme: () => props.theme, label: () => props.label,
    disabled: () => !!props.disabled, search, setSearch, matches, active, setActive, register, select,
    selected: (value) => selectedValue() === value,
    isVisible: (value) => matches().some((entry) => entry.value === value),
    groupHasResults: (group) => matches().some((entry) => group === undefined || entry.group === group),
    emptyLabel: () => props.emptyLabel ?? 'No matching commands.', onKey,
  }
  return createElement(CommandContext.Provider, { value: context },
    <focusScope width="fill" role="group" accessible_name={props.label} enabled={!props.disabled} onKey={onKey}>
      <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border} border_width={1}
        radius={props.theme.overlayRadius} shadow_blur={props.theme.overlayShadowBlur} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={0}>{props.children}</column>
      </rectangle>
    </focusScope>) as ReactElement
}

/** Props for the searchable command input. */
export interface CommandInputProps { /** Search placeholder. */ placeholder?: string }

/** Binds the command query to an accessible native search editor. */
export function ReactCommandInput(props: CommandInputProps = {}): ReactElement {
  const command = useCommand()
  const edits = useRef<InputEditController | null>(null)
  edits.current ??= new InputEditController(command.search())
  return <row width="fill" height={42} gap={8} padding_left={12} padding_right={12} align_items="center">
    <text text="⌕" color={command.theme().muted} font_size={18} accessible_hidden={true} />
    <container grow={1} min_width={0}>
      <textInput nativeKey={`${command.id()}-input`} role="search_input" width="fill" height={28} clip={true}
        value={command.search()} placeholder={props.placeholder ?? 'Type a command or search…'}
        label={`Search ${command.label()}`} enabled={!command.disabled()} background="#00000000"
        text_color={command.theme().foreground} placeholder_color={command.theme().muted}
        caret_color={command.theme().accent} selection_color={selectionTint(command.theme().accent)}
        controls={`${command.id()}-list`} active_descendant={command.active() ? commandItemId(command.id(), command.active()!) : undefined}
        onEdit={(payload) => edits.current!.apply(payload, command.search(), command.setSearch)} />
    </container>
  </row>
}

/** Props for a scrollable option list. */
export interface CommandListProps { /** Command groups, empty state, and separator. */ children: ReactNode }

/** Renders a listbox with the active command exposed as its active descendant. */
export function ReactCommandList(props: CommandListProps): ReactElement {
  const command = useCommand()
  return <focusScope nativeKey={`${command.id()}-list`} role="list_box" accessible_name={command.label()}
    active_descendant={command.active() ? commandItemId(command.id(), command.active()!) : undefined}
    focusable={false}>
    <column width="fill" max_height={280} scroll_y={true} gap={2} padding={4}>{props.children}</column>
  </focusScope>
}

/** Props for an empty-result status. */
export interface CommandEmptyProps { /** Replacement empty-result message. */ text?: string }

/** Announces that the current command query has no matching enabled results. */
export function ReactCommandEmpty(props: CommandEmptyProps = {}): ReactElement {
  const command = useCommand()
  const empty = command.matches().filter((entry) => !entry.disabled).length === 0
  return <row visible={empty} width="fill" height={46} align_items="center" justify_content="center">
    <text text={props.text ?? command.emptyLabel()} color={command.theme().muted} font_size={13} role="status" live="polite" />
  </row>
}

/** Props for a labeled command group. */
export interface CommandGroupProps { /** Optional heading and inherited search group name. */ heading?: string; /** Command rows in this group. */ children: ReactNode }

/** Groups commands and hides the group when filtering leaves it without results. */
export function ReactCommandGroup(props: CommandGroupProps): ReactElement {
  const command = useCommand()
  const visible = command.groupHasResults(props.heading)
  return createElement(CommandGroupContext.Provider, { value: props.heading },
    <focusScope role="group" accessible_name={props.heading} enabled={true} focusable={false}>
      <column width="fill" visible={visible} gap={2}>
        {props.heading ? <text text={props.heading} color={command.theme().muted} font_size={11} weight={600} /> : null}
        {props.children}
      </column>
    </focusScope>) as ReactElement
}

/** Props for one searchable command item. */
export interface CommandItemProps {
  /** Stable unique value submitted when this row is activated. */ value: string
  /** Visible searchable label and accessible name. */ label: string
  /** Optional shortcut shown at the trailing edge. */ shortcut?: string
  /** Whether the command is unavailable. */ disabled?: boolean
  /** Optional row action called after the root selection callback. */ onSelect?: () => void
  /** Custom visual content; defaults to the item label. */ children?: ReactNode
}

/** Registers and renders one pointer- or keyboard-activated command option. */
export function ReactCommandItem(props: CommandItemProps): ReactElement {
  const command = useCommand()
  const group = useContext(CommandGroupContext)
  useEffect(() => command.register({ value: props.value, label: props.label, disabled: !!props.disabled,
    group, onSelect: props.onSelect }), [command.register, props.value, props.label, props.disabled, props.onSelect, group])
  const index = command.matches().findIndex((entry) => entry.value === props.value)
  const active = command.active() === props.value
  const selected = command.selected(props.value)
  const disabled = !!props.disabled || command.disabled()
  return <focusScope nativeKey={commandItemId(command.id(), props.value)} role="option" accessible_name={props.label}
    visible={command.isVisible(props.value)} selected={active} enabled={!disabled}
    accessible_disabled={disabled} position_in_set={index >= 0 ? index + 1 : undefined}
    set_size={command.matches().length || undefined} keyboard_activation="none" onClick={() => command.select(props.value)}
    onSemanticAction={(payload) => { if (payload.action === 'click') command.select(props.value) }}>
    <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => { if (!disabled) command.setActive(props.value) }}>
      <rectangle width="fill" height={34} radius={command.theme().controlRadius}
        background={active ? command.theme().accent : '#00000000'} opacity={disabled ? 0.5 : 1}>
        <row width="fill" height="fill" gap={8} padding_left={8} padding_right={8} align_items="center">
          {selected ? <text text="✓" color={active ? command.theme().accentText : command.theme().accent} font_size={13} />
            : <container width={13} />}
          {props.children ?? <text text={props.label} color={active ? command.theme().accentText : command.theme().foreground}
            font_size={command.theme().controlFontSize} />}
          <container grow={1} />
          {props.shortcut ? <text text={props.shortcut} color={active ? command.theme().accentText : command.theme().muted} font_size={11} /> : null}
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}

/** Props for a visual divider between command groups. */
export interface CommandSeparatorProps { /** Whether this divider is hidden from assistive technology. Defaults to true. */ decorative?: boolean }

/** Draws a horizontal themed divider in the command list. */
export function ReactCommandSeparator(props: CommandSeparatorProps = {}): ReactElement {
  const command = useCommand()
  return <row width="fill" margin_top={3} margin_bottom={3} role="separator" orientation="horizontal"
    accessible_hidden={props.decorative !== false}>
    <rectangle width="fill" height={1} background={command.theme().border} />
  </row>
}

/** Props for a trailing keyboard shortcut label. */
export interface CommandShortcutProps { /** Shortcut text such as ⌘P or Ctrl+K. */ text: string }

/** Renders a muted shortcut hint. */
export function ReactCommandShortcut(props: CommandShortcutProps): ReactElement {
  const command = useCommand()
  return <text text={props.text} color={command.theme().muted} font_size={11} />
}

/** Returns the active Command context or throws when used outside a command root. */
function useCommand(): CommandContextValue {
  const command = useContext(CommandContext)
  if (!command) throw new Error('Command parts must be rendered inside ReactCommand')
  return command
}

/** Returns the stable native target key for one command value. */
function commandItemId(id: string, value: string): string {
  return `${id}-item-${value.replace(/[^a-zA-Z0-9_-]+/g, '-') || 'command'}`
}
