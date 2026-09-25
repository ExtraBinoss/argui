import type { Palette } from './theme'

/** Supported anchored popup placements. */
export type SurfacePlacement =
  | 'top_start' | 'top' | 'top_end'
  | 'bottom_start' | 'bottom' | 'bottom_end'
  | 'left_start' | 'left' | 'left_end'
  | 'right_start' | 'right' | 'right_end'

/** Scrollable axes supported by ScrollArea. */
export type ScrollAreaOrientation = 'vertical' | 'horizontal' | 'both'

/** Props shared by the native Tooltip implementations. */
export interface SurfaceTooltipProps<TTrigger, TContent> {
  /** Stable key for the trigger anchor and popup. */
  id: string
  /** Palette used for the tooltip surface, text, border, and shadow. */
  theme: Palette
  /** Focusable or pointer target which opens the tooltip. */
  children: TTrigger
  /** Native Argui content displayed inside the tooltip. */
  content: TContent
  /** Optional accessible name for the popup and trigger description relationship. */
  description?: string
  /** Controlled popup visibility; omit it to use `defaultOpen`. */
  open?: boolean
  /** Initial popup visibility used only when `open` is uncontrolled. */
  defaultOpen?: boolean
  /** Called when pointer, focus, or popup dismissal requests a visibility change. */
  onOpenChange?: (open: boolean) => void
  /** Popup placement relative to the trigger. Defaults to `top`. */
  placement?: SurfacePlacement
  /** Fixed popup width in logical pixels. Defaults to 220. */
  width?: number
}

/** Props shared by the rich-content HoverCard implementations. */
export interface SurfaceHoverCardProps<TTrigger, TContent> {
  /** Stable key for the trigger anchor and popup. */
  id: string
  /** Accessible name for the hover-card content group. */
  label: string
  /** Palette used for the card surface, border, text, and shadow. */
  theme: Palette
  /** Focusable or pointer target which opens the hover card. */
  children: TTrigger
  /** Rich native Argui content shown inside the hover card. */
  content: TContent
  /** Controlled popup visibility; omit it to use `defaultOpen`. */
  open?: boolean
  /** Initial popup visibility used only when `open` is uncontrolled. */
  defaultOpen?: boolean
  /** Called when pointer, focus, or popup dismissal requests a visibility change. */
  onOpenChange?: (open: boolean) => void
  /** Popup placement relative to the trigger. Defaults to `bottom`. */
  placement?: SurfacePlacement
  /** Fixed popup width in logical pixels. Defaults to 320. */
  width?: number
}

/** Props shared by the Solid and React ScrollArea implementations. */
export interface SurfaceScrollAreaProps<TContent> {
  /** Stable key and accessible name relationship for the viewport. */
  id: string
  /** Accessible name announced for the scrollable region. */
  label: string
  /** Palette used for the viewport, outline, scrollbar, and focus ring. */
  theme: Palette
  /** Native Argui content which may exceed the viewport dimensions. */
  children: TContent
  /** Viewport width in logical pixels. Defaults to 320. */
  width?: number
  /** Viewport height in logical pixels. Defaults to 240. */
  height?: number
  /** Scroll axes enabled in the native viewport. Defaults to vertical. */
  orientation?: ScrollAreaOrientation
  /** Inner padding in logical pixels. Defaults to 12. */
  padding?: number
}

/** Pointer and focus lifecycle callbacks for an anchored, dismissible surface. */
export interface SurfaceActivity {
  /** Opens after the configured pointer-entry delay. */
  triggerEnter(): void
  /** Starts delayed closing when the pointer leaves the trigger. */
  triggerLeave(): void
  /** Keeps the surface open while its popup is hovered. */
  contentEnter(): void
  /** Starts delayed closing when the pointer leaves the popup. */
  contentLeave(): void
  /** Opens immediately while a descendant is focused. */
  focusEnter(): void
  /** Starts delayed closing after focus leaves the surface. */
  focusLeave(): void
  /** Cancels timers and closes after an explicit native dismissal. */
  dismiss(): void
  /** Releases pending timers when the owning widget is removed. */
  dispose(): void
}

/** Creates coordinated hover and focus handling for a popup anchored to a trigger. */
export function createSurfaceActivity(
  onOpenChange: (open: boolean) => void,
  openDelay: number,
  closeDelay: number,
): SurfaceActivity {
  let triggerHovered = false
  let contentHovered = false
  let focused = false
  let openTimer: ReturnType<typeof setTimeout> | undefined
  let closeTimer: ReturnType<typeof setTimeout> | undefined

  const cancelTimers = () => {
    if (openTimer !== undefined) clearTimeout(openTimer)
    if (closeTimer !== undefined) clearTimeout(closeTimer)
    openTimer = undefined
    closeTimer = undefined
  }
  const scheduleClose = () => {
    if (openTimer !== undefined) clearTimeout(openTimer)
    openTimer = undefined
    if (closeTimer !== undefined) clearTimeout(closeTimer)
    closeTimer = setTimeout(() => {
      closeTimer = undefined
      if (!triggerHovered && !contentHovered && !focused) onOpenChange(false)
    }, Math.max(0, closeDelay))
  }

  return {
    triggerEnter() {
      triggerHovered = true
      if (closeTimer !== undefined) clearTimeout(closeTimer)
      closeTimer = undefined
      if (openTimer !== undefined) clearTimeout(openTimer)
      if (openDelay <= 0) {
        openTimer = undefined
        onOpenChange(true)
      } else {
        openTimer = setTimeout(() => {
          openTimer = undefined
          if (triggerHovered) onOpenChange(true)
        }, openDelay)
      }
    },
    triggerLeave() {
      triggerHovered = false
      scheduleClose()
    },
    contentEnter() {
      contentHovered = true
      cancelTimers()
      onOpenChange(true)
    },
    contentLeave() {
      contentHovered = false
      scheduleClose()
    },
    focusEnter() {
      focused = true
      cancelTimers()
      onOpenChange(true)
    },
    focusLeave() {
      focused = false
      scheduleClose()
    },
    dismiss() {
      triggerHovered = false
      contentHovered = false
      focused = false
      cancelTimers()
      onOpenChange(false)
    },
    dispose() {
      cancelTimers()
    },
  }
}
