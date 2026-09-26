import type { NativeBridge, ThemeBridge, ThemeWireSnapshot, ThemeWireValue } from './protocol'

/** Token values supported by the shared native theme wire format. */
export type ThemeValues = Record<string, ThemeWireValue>

/** Smallest engine phase affected by a token change. */
export type ThemeImpact = 'Semantics' | 'Composite' | 'Paint' | 'Scroll' | 'Layout'

/** The `argui-theme` type assigned to a declarative token. */
export type ThemeTokenType =
  | 'Color' | 'Brush' | 'Float' | 'Int' | 'Bool' | 'Length'
  | 'Percentage' | 'Duration' | 'Angle' | 'FontFamily'
  | 'FontWeight' | 'FontSize' | 'LineHeight'

/** One typed token definition passed to the host without local resolution. */
export interface ThemeTokenDefinition<T extends ThemeWireValue> {
  type: ThemeTokenType
  default: T
  impact?: ThemeImpact
}

/** Application theme schema, named variants, and system appearance mapping. */
export interface ThemeDefinition<T extends object> {
  tokens: { [K in keyof T]: T[K] extends ThemeWireValue ? ThemeTokenDefinition<T[K]> : never }
  variants?: Record<string, Partial<T>>
  initialVariant?: string
  systemVariants?: { light: string; dark: string }
}

/** One atomic mutation of the application theme. Omitted fields remain intact. */
export interface ThemePatch<T extends object> {
  variant?: string | null
  overrides?: Partial<T>
  removeOverrides?: (keyof T & string)[]
  systemScheme?: 'light' | 'dark'
}

/** Resolved values and token revisions owned by the host theme engine. */
export interface ThemeSnapshot<T extends object> {
  readonly revision: number
  readonly variant: string | null
  readonly resolvedVariant: string | null
  readonly systemScheme: 'light' | 'dark'
  readonly values: Readonly<T>
  readonly tokenRevisions: Readonly<Record<keyof T & string, number>>
}

/** Tokens whose resolved values changed during one atomic host update. */
export interface ThemeChange<T extends object> {
  readonly tokens: readonly (keyof T & string)[]
  readonly impact: ThemeImpact | null
}

/** Resolves sparse subtree overrides against one coherent native theme snapshot. */
export function mergeThemeOverrides<T extends object>(base: Readonly<T>, overrides: Partial<T>): Readonly<T> {
  const merged = { ...base }
  for (const [key, value] of Object.entries(overrides)) {
    if (value === undefined) continue
    if (!(key in base)) throw new Error(`Unknown theme token: ${key}`)
    if (typeof value !== typeof (base as Record<string, unknown>)[key]) {
      throw new TypeError(`Invalid theme token type: ${key}`)
    }
    Object.assign(merged, { [key]: value })
  }
  return Object.freeze(merged)
}

type ThemeListener<T extends object> = (snapshot: ThemeSnapshot<T>, change: ThemeChange<T>) => void

/** One app or window theme session; resolution remains in the host's `argui-theme` runtime. */
export class ThemeRuntime<T extends object> {
  private readonly bridge: ThemeBridge
  private readonly id: number
  private readonly unsubscribeHost: () => void
  private readonly listeners = new Set<{ listener: ThemeListener<T>; keys?: ReadonlySet<string> }>()
  private current: ThemeSnapshot<T>
  private disposed = false

  /** Creates a native theme session from `definition` on `bridge`. */
  constructor(bridge: NativeBridge, definition: ThemeDefinition<T>) {
    if (!bridge.theme) throw new Error('Argui host does not expose the theme runtime')
    this.bridge = bridge.theme
    const created = this.bridge.create(definition)
    this.id = created.id
    this.current = this.readSnapshot(created.snapshot)
    this.unsubscribeHost = this.bridge.subscribe(this.id, (snapshot) => this.receive(snapshot))
  }

  /** Returns the complete, cached host revision without a native token read. */
  snapshot(): ThemeSnapshot<T> { return this.current }

  /** Applies `patch` atomically and returns the resulting coherent snapshot. */
  update(patch: ThemePatch<T>): ThemeSnapshot<T> {
    if (this.disposed) throw new Error('Theme runtime is disposed')
    this.receive(this.bridge.update(this.id, patch))
    return this.current
  }

  /** Subscribes to revisions; `keys` limits callbacks to changed token names. */
  subscribe(listener: ThemeListener<T>, keys?: readonly (keyof T & string)[]): () => void {
    if (this.disposed) throw new Error('Theme runtime is disposed')
    const entry = { listener, keys: keys ? new Set<string>(keys) : undefined }
    this.listeners.add(entry)
    return () => { this.listeners.delete(entry) }
  }

  /** Releases the native theme session and its subscriber. */
  dispose(): void {
    if (this.disposed) return
    this.disposed = true
    this.unsubscribeHost()
    this.listeners.clear()
    this.bridge.dispose(this.id)
  }

  private readSnapshot(wire: ThemeWireSnapshot): ThemeSnapshot<T> {
    return Object.freeze({
      revision: wire.revision,
      variant: wire.variant,
      resolvedVariant: wire.resolvedVariant,
      systemScheme: wire.systemScheme,
      values: Object.freeze({ ...wire.values }) as T,
      tokenRevisions: Object.freeze({ ...wire.tokenRevisions }) as Record<keyof T & string, number>,
    })
  }

  private receive(wire: ThemeWireSnapshot): void {
    if (this.disposed || wire.revision <= this.current.revision) return
    const previous = this.current
    const next = this.readSnapshot(wire)
    const tokens = wire.change?.tokens ?? Object.keys(next.tokenRevisions).filter(
      (key) => (next.tokenRevisions as Record<string, number>)[key]
        !== (previous.tokenRevisions as Record<string, number>)[key],
    )
    const change: ThemeChange<T> = {
      tokens: tokens as (keyof T & string)[],
      impact: wire.change?.impact ?? null,
    }
    this.current = next
    for (const { listener, keys } of [...this.listeners]) {
      if (!keys || change.tokens.some((token) => keys.has(token))) listener(next, change)
    }
  }
}

/** Creates a theme session scoped to the supplied application bridge. */
export function createThemeRuntime<T extends object>(
  bridge: NativeBridge,
  definition: ThemeDefinition<T>,
): ThemeRuntime<T> {
  return new ThemeRuntime(bridge, definition)
}
