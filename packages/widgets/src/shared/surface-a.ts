/** Value accepted by a single or multiple accordion root. */
export type SurfaceAAccordionValue = string | readonly string[]

/** Orientation used by the tabs composite. */
export type SurfaceATabsOrientation = 'horizontal' | 'vertical'

/** Visual style of a tabs list. */
export type SurfaceATabsVariant = 'default' | 'line'

/** Keyboard activation policy for tabs. */
export type SurfaceATabsActivationMode = 'automatic' | 'manual'

/** Encodes arbitrary component values into stable, collision-resistant ID parts. */
export function surfaceAIdPart(value: string): string {
  const encoded = Array.from(value, (character) => character.codePointAt(0)!.toString(16)).join('-')
  return encoded || 'empty'
}

/** Returns a pressed key name from a native key event payload. */
export function surfaceAKey(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (typeof payload !== 'object' || payload === null) return undefined
  const event = payload as Record<string, unknown>
  if (event.state !== undefined && event.state !== 'pressed') return undefined
  return typeof event.key === 'string' ? event.key : undefined
}
