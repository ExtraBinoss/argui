import type { AssetRef, SemanticCurrent } from '@argui/host'
import type { Palette } from './theme'
import type { ButtonSize } from './button-size'
import type { SelectOption } from './select-options'

/** Icons injected by an application instead of loaded from its asset catalogue. */
export interface WidgetIcons {
  search?: AssetRef
  check?: AssetRef
  x?: AssetRef
  chevronDown?: AssetRef
  chevronRight?: AssetRef
  loader?: AssetRef
}

/** Props shared by the native button implementations. */
export interface ButtonProps {
  id: string
  label: string
  /** Spoken button name when the visible label is abbreviated. */
  accessibleLabel?: string
  theme: Palette
  onClick: () => void
  disabled?: boolean
  busy?: boolean
  selected?: boolean
  size?: ButtonSize
  role?: 'button' | 'switch'
  current?: SemanticCurrent
  kind?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive' | 'quiet' | 'link'
  icon?: AssetRef
  /** Clockwise icon rotation in degrees. */
  iconRotation?: number
  activeIcon?: AssetRef
  iconOnly?: boolean
  expanded?: boolean
  controls?: string
}

/** Props shared by the native selection popup implementations. */
export interface SelectProps {
  id: string
  label: string
  options: readonly SelectOption[]
  value?: string
  defaultValue?: string
  theme: Palette
  onChange?: (value: string) => void
  placeholder?: string
  disabled?: boolean
  width?: number
}

/** Props shared by the themed native text input implementations. */
export interface InputFieldProps {
  id: string
  label: string
  theme: Palette
  value?: string
  defaultValue?: string
  onChange?: (value: string) => void
  placeholder?: string
  disabled?: boolean
  readOnly?: boolean
  invalid?: boolean
  search?: boolean
  password?: boolean
  selectionColor?: string
  showLabel?: boolean
  onSubmit?: (value: string) => void
}

/** Props shared by the anchored native popover implementations. */
export interface PopoverProps<TChildren> {
  id: string
  label: string
  theme: Palette
  blur?: boolean
  opaque?: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
  placement?: 'top_start' | 'top' | 'top_end' | 'bottom_start' | 'bottom' | 'bottom_end' | 'left_start' | 'left' | 'left_end' | 'right_start' | 'right' | 'right_end'
  width?: number
  /** Omit the built-in button when an external control with `id` anchors the popover. */
  trigger?: false
  /** Native key to focus when the popover opens; defaults to its first focusable child. */
  initialFocus?: string
  /** Padding around custom popover content; defaults to the theme overlay padding. */
  contentPadding?: number
  /** Mount a bounded native virtual list for fixed-height popup items. */
  virtualItems?: {
    /** Total number of popup rows. */ count: number
    /** Fixed row height in logical pixels. */ itemHeight: number
    /** Maximum visible list height in logical pixels. */ height: number
    /** First row shown when the popup opens. */ initialIndex?: number
    /** Stable key for each row; defaults to its index. */ itemKey?: (index: number) => string | number
    /** Render one visible row. */ renderItem: (index: number) => TChildren
  }
  closeLabel?: string | false
  children?: TChildren
}

/** Props shared by centered, accessible native modal dialogs. */
export interface DialogProps<TChildren> {
  id: string
  title: string
  description?: string
  open: boolean
  onOpenChange: (open: boolean) => void
  theme: Palette
  children: TChildren
  width?: number
  radius?: number
  padding?: number
  blur?: number
  scrimColor?: string
  surfaceColor?: string
  closeLabel?: string | false
}
