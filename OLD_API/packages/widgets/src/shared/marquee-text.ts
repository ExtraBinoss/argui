import type { Palette } from './theme'

/** Options for measured single-line text that scrolls only when it overflows. */
export interface MarqueeTextProps {
  /** Stable native identity for the measured viewport. */ id: string
  /** Full visible and accessible text. */ text: string
  /** Width of the clipped text viewport in logical pixels. */ width: number
  /** Theme supplying text and trailing-shadow colors. */ theme: Palette
  /** Text size in logical pixels. Defaults to the theme control size. */ fontSize?: number
  /** Scroll speed in logical pixels per second. Defaults to 45. */ speed?: number
  /** Pause the native motion when the pointer leaves. Defaults to true. */ pauseOnLeave?: boolean
  /** Hold briefly at the far end before the native motion returns. */ holdAtEnd?: boolean
  /** Width of each enabled overflow shadow. Defaults to 18. */ shadowWidth?: number
  /** Optional tinted edge color; omission fades into the underlying surface or blur. */ shadowColor?: string
  /** Shadow strength from zero to one. Defaults to one. */ shadowIntensity?: number
  /** Enable the leading edge shadow when the viewport is scrolled. Defaults to false. */ shadowStart?: boolean
  /** Enable the trailing edge shadow while text overflows. Defaults to true. */ shadowEnd?: boolean
  /** Keep the trailing shadow visible while the marquee moves. Defaults to false. */ shadowOnHover?: boolean
  /** Optional text shown over the trailing edge only while content overflows; accepts any string. */ overflowMarker?: string
}

/** Reads a native virtual item's measured extent and its viewport width. */
export function marqueeMeasurement(payload: unknown): { content: number; viewport: number } | undefined {
  if (!payload || typeof payload !== 'object') return undefined
  const value = payload as { items?: Array<{ index?: unknown; extent?: unknown }>; viewportExtent?: unknown }
  const content = value.items?.find((item) => item.index === 0)?.extent
  const viewport = value.viewportExtent
  return typeof content === 'number' && Number.isFinite(content) && content >= 0
    && typeof viewport === 'number' && Number.isFinite(viewport) && viewport >= 0
    ? { content, viewport } : undefined
}
