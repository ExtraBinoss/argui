import type { Palette } from './theme'

/** Viewport edge used to position a sheet or drawer panel. */
export type SurfacePanelSide = 'top' | 'right' | 'bottom' | 'left'

/** Props shared by the controlled or uncontrolled AlertDialog widgets. */
export interface AlertDialogProps {
  /** Stable key for the trigger, modal window, and confirmation actions. */
  id: string
  /** Accessible dialog title. */
  title: string
  /** Explanation announced with the dialog title. */
  description: string
  /** Text shown on the button which opens the dialog. */
  triggerLabel: string
  /** Label shown on the confirmation action. */
  confirmLabel: string
  /** Label shown on the safe cancel action. Defaults to `Cancel`. */
  cancelLabel?: string
  /** Palette used for the scrim, dialog surface, text, and buttons. */
  theme: Palette
  /** Called after the user confirms the described action. */
  onConfirm: () => void
  /** Called when the user cancels with the cancel button or Escape. */
  onCancel?: () => void
  /** Whether to emphasize the confirmation action as destructive. Defaults to true. */
  destructive?: boolean
  /** Whether the confirmation action is disabled. */
  confirmDisabled?: boolean
  /** Controlled popup visibility; omit it to use `defaultOpen`. */
  open?: boolean
  /** Initial popup visibility used only when `open` is uncontrolled. */
  defaultOpen?: boolean
  /** Called when the trigger, cancel, or dismissal requests a visibility change. */
  onOpenChange?: (open: boolean) => void
  /** Dialog width in logical pixels. Defaults to 440. */
  width?: number
}

/** Props shared by the side-positioned Sheet and Drawer widgets. */
export interface SurfacePanelProps<TContent> {
  /** Stable key for the trigger and modal panel. */
  id: string
  /** Accessible name announced for the panel. */
  title: string
  /** Optional explanation rendered below the title. */
  description?: string
  /** Text shown on the button which opens the panel. */
  triggerLabel: string
  /** Native Argui content displayed inside the panel. */
  content: TContent
  /** Palette used for the scrim, panel surface, text, and close button. */
  theme: Palette
  /** Viewport side from which the panel is placed. Defaults per widget. */
  side?: SurfacePanelSide
  /** Width for left and right panels in logical pixels. Defaults to 420. */
  width?: number
  /** Height for top and bottom panels in logical pixels. Defaults to 360. */
  height?: number
  /** Panel body padding in logical pixels. Defaults to theme overlay padding. */
  padding?: number
  /** Text shown on the panel's close button; set false to omit that button. */
  closeLabel?: string | false
  /** Controlled popup visibility; omit it to use `defaultOpen`. */
  open?: boolean
  /** Initial popup visibility used only when `open` is uncontrolled. */
  defaultOpen?: boolean
  /** Called when the trigger, close button, or dismissal requests a visibility change. */
  onOpenChange?: (open: boolean) => void
}

/** Layout values for a panel aligned to one viewport edge. */
export interface SurfacePanelBounds {
  /** Panel width or fill dimension. */
  width: number | 'fill'
  /** Panel height or fill dimension. */
  height: number | 'fill'
  /** Left edge inset in logical pixels. */
  insetLeft?: number
  /** Right edge inset in logical pixels. */
  insetRight?: number
  /** Top edge inset in logical pixels. */
  insetTop?: number
  /** Bottom edge inset in logical pixels. */
  insetBottom?: number
}

/** Computes edge constraints for a panel positioned in a full-viewport modal. */
export function surfacePanelBounds(
  side: SurfacePanelSide,
  width: number,
  height: number,
): SurfacePanelBounds {
  if (side === 'top') return { width: 'fill', height, insetLeft: 0, insetRight: 0, insetTop: 0 }
  if (side === 'bottom') return { width: 'fill', height, insetLeft: 0, insetRight: 0, insetBottom: 0 }
  if (side === 'left') return { width, height: 'fill', insetLeft: 0, insetTop: 0, insetBottom: 0 }
  return { width, height: 'fill', insetRight: 0, insetTop: 0, insetBottom: 0 }
}

/** Extracts a finite logical-pixel displacement along `axis` from a drag callback. */
export function surfaceDragDelta(payload: unknown, axis: 'x' | 'y'): number {
  if (typeof payload === 'number' && Number.isFinite(payload)) return payload
  if (typeof payload === 'object' && payload !== null) {
    const values = payload as Record<string, unknown>
    const delta = values[axis === 'x' ? 'deltaX' : 'deltaY'] ?? values.delta
    if (typeof delta === 'number' && Number.isFinite(delta)) return delta
  }
  return 0
}

/** Uses the native total displacement when available, otherwise accumulates `previous`. */
export function surfaceDragDistance(payload: unknown, axis: 'x' | 'y', previous: number): number {
  if (typeof payload === 'object' && payload !== null) {
    const total = (payload as Record<string, unknown>)[axis === 'x' ? 'totalX' : 'totalY']
    if (typeof total === 'number' && Number.isFinite(total)) return total
  }
  return previous + surfaceDragDelta(payload, axis)
}

/** Returns whether the native pan has finished and its visual offset must clear. */
export function surfaceDragFinished(payload: unknown): boolean {
  if (typeof payload !== 'object' || payload === null) return false
  const phase = (payload as Record<string, unknown>).phase
  return phase === 'ended' || phase === 'cancelled'
}

/** Projects a released pan briefly using native velocity to recognize a flick. */
export function surfaceDragProjected(payload: unknown, axis: 'x' | 'y', distance: number): number {
  if (typeof payload !== 'object' || payload === null) return distance
  const velocity = (payload as Record<string, unknown>)[axis === 'x' ? 'velocityX' : 'velocityY']
  return typeof velocity === 'number' && Number.isFinite(velocity) ? distance + velocity * 0.18 : distance
}
