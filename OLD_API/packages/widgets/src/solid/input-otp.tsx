import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createContext, useContext, type JSX as SolidJSX } from 'solid-js'
import { InputEditController } from '../shared/input-edit'
import { inputOtpLength, normalizeInputOtpValue } from '../shared/foundation-h'
import type { Palette } from '../shared/theme'

interface InputOTPContextValue {
  value(): string
  focused(): boolean
  activeIndex(): number
  length(): number
  cellSize(): number
  theme(): Palette
  disabled(): boolean
  invalid(): boolean
}

const InputOTPContext = createContext<InputOTPContextValue>()

/** Props for a digit-only segmented one-time-password editor. */
export interface InputOTPProps {
  /** Stable key used to retain the native editor. */
  id: string
  /** Palette used for cells, input text, and focus state. */
  theme: Palette
  /** Accessible name announced for the actual native text editor. */
  label: string
  /** Maximum number of ASCII digits; defaults to six. */
  maxLength?: number
  /** Controlled digit string. Omit to use `defaultValue` and internal state. */
  value?: string
  /** Initial digit string for an uncontrolled editor. */
  defaultValue?: string
  /** Called with the complete digit string after each edit. */
  onChange?: (value: string) => void
  /** Supporting hint announced by the native editor. */
  description?: string
  /** Optional visible-label key referenced by the native editor. */
  labelledBy?: string
  /** Whether the native editor is disabled. */
  disabled?: boolean
  /** Whether this code is required. */
  required?: boolean
  /** Whether the code is invalid. */
  invalid?: boolean
  /** Whether the current code is visible but cannot be edited. */
  readOnly?: boolean
  /** Height of the segmented visual track. */
  cellSize?: number
  /** Space between custom slot groups. */
  gap?: number
  /** Slot groups, separators, and slots. */
  children: JSX.Element
}

/** Renders one native digit editor behind user-composed, accessible OTP cells. */
export function InputOTP(props: InputOTPProps): JSX.Element {
  const length = () => inputOtpLength(props.maxLength)
  const cellSize = () => Number.isFinite(props.cellSize) && props.cellSize! > 0 ? props.cellSize! : 40
  const [uncontrolled, setUncontrolled] = createSignal(normalizeInputOtpValue(props.defaultValue ?? '', length()))
  const [focused, setFocused] = createSignal(false)
  const [activeIndex, setActiveIndex] = createSignal(0)
  const value = () => normalizeInputOtpValue(props.value !== undefined ? props.value : uncontrolled(), length())
  const edits = new InputEditController(value())
  const content = (
    <container width="fill" height={cellSize()} position="relative">
      <row width="fill" height="fill" gap={props.gap ?? 8} align_items="center">{props.children}</row>
      <container position="absolute" inset_left={0} inset_right={0} inset_top={0} inset_bottom={0}>
        <textInput key={props.id} role="text_input" width="fill" height="fill" clip={true}
          value={value()} label={props.label} description={props.description} labelled_by={props.labelledBy}
          enabled={!props.disabled} read_only={!!props.readOnly} required={!!props.required} invalid={!!props.invalid}
          max_digits={length()} background="#00000000" text_color="#00000000" placeholder_color="#00000000"
          caret_color="#00000000" caret_fill="#00000000" selection_color="#00000000"
          onFocus={() => { setFocused(true); setActiveIndex(Math.min(value().length, length() - 1)) }}
          onBlur={() => setFocused(false)}
          onEdit={(payload) => edits.apply(payload, value(), (next) => {
            if (props.value === undefined) setUncontrolled(next)
            props.onChange?.(next)
            setActiveIndex(Math.min(next.length, length() - 1))
          })} />
      </container>
    </container>
  ) as SolidJSX.Element
  const context: InputOTPContextValue = {
    value, focused: () => focused(), activeIndex: () => activeIndex(), length,
    cellSize, theme: () => props.theme, disabled: () => !!props.disabled, invalid: () => !!props.invalid,
  }
  return <InputOTPContext.Provider value={context}>{content}</InputOTPContext.Provider>
}

/** Props for a horizontal row that groups adjacent OTP slots. */
export interface InputOTPGroupProps {
  /** Adjacent OTP slot children. */
  children: JSX.Element
  /** Gap between cells within this group. Defaults to zero. */
  gap?: number
}

/** Groups adjacent OTP cells into a single visual run. */
export function InputOTPGroup(props: InputOTPGroupProps): JSX.Element {
  return <row gap={props.gap ?? 4} align_items="center">{props.children}</row>
}

/** Props for one indexed character cell in the segmented input. */
export interface InputOTPSlotProps {
  /** Zero-based digit position represented by this cell. */
  index: number
  /** Optional cell size override in logical pixels. */
  size?: number
}

/** Shows one digit and a focus indicator for the current insertion position. */
export function InputOTPSlot(props: InputOTPSlotProps): JSX.Element | null {
  const context = useContext(InputOTPContext)
  if (!context) return null
  const valid = Number.isInteger(props.index) && props.index >= 0 && props.index < context.length()
  const character = () => valid ? context.value()[props.index] ?? '' : ''
  const active = () => valid && context.focused() && context.activeIndex() === props.index && !context.disabled()
  const theme = () => context.theme()
  const size = () => props.size ?? context.cellSize()
  const invalid = () => context.invalid()
  return <rectangle width={size()} height={size()} background={theme().surface}
    border_color={invalid() ? theme().destructive : active() ? theme().accent : theme().border}
    border_width={active() ? 2 : 1} radius={Math.max(4, theme().controlRadius / 2)}
    accessible_hidden={true}>
    <row width="fill" height="fill" align_items="center" justify_content="center">
      {character() ? <text text={character()} color={theme().foreground}
        font_size={theme().controlFontSize} weight={500} accessible_hidden={true} />
        : active() ? <rectangle width={2} height={Math.min(18, size() * 0.48)}
          radius={1} background={theme().accent} accessible_hidden={true} /> : null}
    </row>
  </rectangle>
}

/** Props for a semantic separator between OTP groups. */
export interface InputOTPSeparatorProps {
  /** Separator glyph; defaults to an en dash. */
  text?: string
  /** Palette used for the separator. */
  theme?: Palette
}

/** Marks a visual break between OTP cell groups. */
export function InputOTPSeparator(props: InputOTPSeparatorProps): JSX.Element {
  const context = useContext(InputOTPContext)
  return <text text={props.text ?? '–'} color={props.theme?.muted ?? context?.theme().muted ?? '#596377'}
    role="separator" orientation="vertical" accessible_name="Digit group separator" />
}
