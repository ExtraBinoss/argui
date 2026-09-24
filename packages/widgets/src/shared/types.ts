import type { AssetRef, SemanticCurrent } from '@argui/host'
import type { Palette } from './theme'

/** Icons injected by an application instead of loaded from its asset catalogue. */
export interface WidgetIcons {
  search?: AssetRef
  chevronDown?: AssetRef
  loader?: AssetRef
}

/** Props shared by the native button implementations. */
export interface ButtonProps {
  id: string
  label: string
  theme: Palette
  onClick: () => void
  disabled?: boolean
  busy?: boolean
  selected?: boolean
  role?: 'button' | 'switch'
  current?: SemanticCurrent
  kind?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive' | 'quiet'
  icon?: AssetRef
  activeIcon?: AssetRef
  iconOnly?: boolean
  expanded?: boolean
  controls?: string
}

/** Props shared by the controlled native selector implementations. */
export interface SelectProps {
  id: string
  label: string
  options: readonly string[]
  value: string
  theme: Palette
  onChange: (value: string) => void
}

/** Props shared by the themed native text input implementations. */
export interface InputFieldProps {
  id: string
  label: string
  theme: Palette
  value: string
  onChange: (value: string) => void
  placeholder?: string
  disabled?: boolean
  readOnly?: boolean
  invalid?: boolean
  search?: boolean
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
  children: TChildren
}
