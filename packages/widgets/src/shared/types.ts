import type { ConstraintValue, DimensionValue, InsetsValue } from '@argui/host'

/** Self-alignment values supported by native flex layout. */
export type WidgetAlignSelf = 'start' | 'center' | 'end' | 'stretch'

/** Layout properties accepted by each widget's outer native node. */
export interface WidgetLayoutProps {
  /** Optional native identity for anchors, relations, and tests. */
  id?: string
  /** Preferred outer width in logical pixels, percent, or intrinsic sizing. */
  width?: DimensionValue
  /** Preferred outer height in logical pixels, percent, or intrinsic sizing. */
  height?: DimensionValue
  /** Minimum outer width in logical pixels, percent, or intrinsic sizing. */
  minWidth?: ConstraintValue
  /** Maximum outer width in logical pixels, percent, or intrinsic sizing. */
  maxWidth?: ConstraintValue
  /** Minimum outer height in logical pixels, percent, or intrinsic sizing. */
  minHeight?: ConstraintValue
  /** Maximum outer height in logical pixels, percent, or intrinsic sizing. */
  maxHeight?: ConstraintValue
  /** Flex growth when the parent has remaining space. */
  grow?: number
  /** Flex shrink when the parent is smaller than the preferred size. */
  shrink?: number
  /** Cross-axis alignment within the parent flex container. */
  alignSelf?: WidgetAlignSelf
  /** Space outside the widget root. */
  margin?: InsetsValue
}

/** Presentation choices shared by both Button adapters. */
export type ButtonVariant = 'default' | 'outline' | 'secondary' | 'ghost' | 'destructive' | 'link'

/** Sizes used by the shadcn-style button surface. */
export type ButtonSize = 'default' | 'xs' | 'sm' | 'lg' | 'icon' | 'icon-xs' | 'icon-sm' | 'icon-lg'

/** Sizes that render an icon-sized square and therefore need an accessible name. */
export type ButtonIconSize = Extract<ButtonSize, `icon${string}`>

/** Props shared by Solid and React buttons, excluding framework-specific children. */
export type ButtonOptions = WidgetLayoutProps & {
  variant?: ButtonVariant
  /** Whether a pointer press gently scales the button; enabled by default. */
  pressAnimation?: boolean
  /** Horizontal alignment of the button content. */
  contentAlign?: 'start' | 'center' | 'end'
  /** Pressed state exposed to assistive technology for a toggle-style button. */
  pressed?: boolean
  disabled?: boolean
  onClick: () => void
  iconOnly?: boolean
  expanded?: boolean
  controls?: string
} & (
  | { iconOnly: true; size?: ButtonSize; accessibleName: string }
  | { iconOnly?: false; size: ButtonIconSize; accessibleName: string }
  | { iconOnly?: false; size?: Exclude<ButtonSize, ButtonIconSize>; accessibleName?: string }
)

/** Shared options for a labelled group of adjacent controls. */
export type ButtonGroupOptions = WidgetLayoutProps & {
  accessibleName: string
  orientation?: 'horizontal' | 'vertical'
  directionScope?: 'ltr' | 'rtl'
}

/** Props shared by both native text input implementations. */
export type InputFieldOptions = WidgetLayoutProps & {
  value?: string
  defaultValue?: string
  onValueChange?: (value: string) => void
  onSubmit?: (value: string) => void
  placeholder?: string
  type?: 'text' | 'search' | 'password'
  disabled?: boolean
  readOnly?: boolean
  invalid?: boolean
} & (
  | { label: string; accessibleName?: string }
  | { label?: undefined; accessibleName: string }
)

/** Props shared by both popover implementations, excluding framework children. */
export type PopoverOptions = WidgetLayoutProps & {
  trigger: string
  /** Accessible name for the trigger and popup; defaults to the visible trigger text. */
  accessibleLabel?: string
  placement?: 'topStart' | 'top' | 'topEnd' | 'bottomStart' | 'bottom' | 'bottomEnd'
    | 'leftStart' | 'left' | 'leftEnd' | 'rightStart' | 'right' | 'rightEnd'
  /** Preferred popup width, independent of the trigger and outer layout. */
  contentWidth?: DimensionValue
  blur?: boolean
  opaque?: boolean
  closeLabel?: string | false
  /** Focus the first popup control, or leave focus on the trigger. */
  initialFocus?: 'first' | 'trigger'
} & (
  | { open: boolean; onOpenChange: (open: boolean) => void; defaultOpen?: never }
  | { open?: never; onOpenChange?: (open: boolean) => void; defaultOpen?: boolean }
)

/** Native option rendered by Select. Values must be unique within one Select. */
export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

/** Visual presentations supported by both Select adapters. */
export type SelectVariant = 'default' | 'shadcn'

/** Shared options for the Solid and React Select implementations. */
export interface SelectOptions extends WidgetLayoutProps {
  /** Standard field or compact trigger with its group title inside the open menu. */
  variant?: SelectVariant
  options: readonly SelectOption[]
  value?: string
  defaultValue?: string
  onValueChange?: (value: string) => void
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
  label: string
  placeholder?: string
  disabled?: boolean
}

/** Keyboard event names accepted by the native select's key handler. */
export function keyName(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (payload && typeof payload === 'object' && 'state' in payload && payload.state !== 'pressed') return undefined
  if (payload && typeof payload === 'object' && 'key' in payload && typeof payload.key === 'string') {
    return payload.key
  }
  return undefined
}

/** Finds the next enabled option, wrapping at either end. */
export function nextEnabledOption(options: readonly SelectOption[], current: number, direction: 1 | -1): number {
  for (let step = 1; step <= options.length; step++) {
    const index = (current + direction * step + options.length) % options.length
    if (!options[index]?.disabled) return index
  }
  return -1
}
