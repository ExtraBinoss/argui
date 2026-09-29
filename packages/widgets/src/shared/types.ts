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
export type ButtonOptions = Omit<WidgetLayoutProps, 'height' | 'minHeight' | 'maxHeight'> & {
  variant?: ButtonVariant
  /** Whether a pointer press moves the button down one pixel; enabled by default. */
  pressAnimation?: boolean
  /** Horizontal alignment of the button content. */
  contentAlign?: 'start' | 'center' | 'end'
  /** Use pill corners on an ungrouped button. */
  rounded?: boolean
  /** Overrides the size-dependent corner radius on an ungrouped non-pill button. */
  radius?: number
  /** Controlled selected state of a toggle button, exposed as pressed to assistive technology. */
  pressed?: boolean
  disabled?: boolean
  onClick: () => void
  iconOnly?: boolean
  expanded?: boolean
  controls?: string
  /** Popup role advertised by this button; set by the component that owns the popup. */
  hasPopup?: 'menu' | 'listBox' | 'dialog' | 'grid' | 'tree'
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

/** Controlled, read-only, or autonomous value ownership for a text field. */
type InputValueOptions =
  | { value: string; onValueChange: (value: string) => void; defaultValue?: never; readOnly?: boolean }
  | { value: string; onValueChange?: never; defaultValue?: never; readOnly: true }
  | { value?: never; defaultValue?: string; onValueChange?: (value: string) => void; readOnly?: boolean }

/** Props shared by both native text input implementations. */
export type InputFieldOptions = WidgetLayoutProps & InputValueOptions & {
  onSubmit?: (value: string) => void
  placeholder?: string
  type?: 'text' | 'search' | 'password'
  disabled?: boolean
  invalid?: boolean
  /** Accessible help text announced with the native editor. */
  description?: string
  /** Exposes a required value to assistive technology. */
  required?: boolean
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
  /** Prefer a native popup surface that may extend beyond the application window. */
  allowOutsideWindow?: boolean
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

/** Shared options for a hover-triggered, informational tooltip. */
export type TooltipOptions = WidgetLayoutProps & {
  /** Text shown in the tooltip and announced to assistive technology. */
  content: string
  placement?: 'topStart' | 'top' | 'topEnd' | 'bottomStart' | 'bottom' | 'bottomEnd'
    | 'leftStart' | 'left' | 'leftEnd' | 'rightStart' | 'right' | 'rightEnd'
  contentWidth?: DimensionValue
  /** Prefer a native tooltip surface that may extend beyond the application window. */
  allowOutsideWindow?: boolean
  blur?: boolean
  opaque?: boolean
}

/** Visual presentations supported by both Select adapters. */
export type SelectVariant = 'default' | 'shadcn'

/** Shared options for the Solid and React Select implementations. */
export type SelectOptions = WidgetLayoutProps & {
  /** Popup width in UI pixels or another native dimension, independent of the field. */
  contentWidth?: DimensionValue
  /** Standard field or compact trigger with its group title inside the open menu. */
  variant?: SelectVariant
  options: readonly SelectOption[]
  /** Prefer a native option surface that may extend beyond the application window. */
  allowOutsideWindow?: boolean
  label: string
  placeholder?: string
  /** Whether a compact menu includes a selectable empty placeholder; defaults to true. */
  allowClear?: boolean
  disabled?: boolean
} & (
  | { value: string; onValueChange?: (value: string) => void; defaultValue?: never }
  | { value?: never; defaultValue?: string; onValueChange?: (value: string) => void }
) & (
  | { open: boolean; onOpenChange: (open: boolean) => void; defaultOpen?: never }
  | { open?: never; defaultOpen?: boolean; onOpenChange?: (open: boolean) => void }
)

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
