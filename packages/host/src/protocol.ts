/** An identity owned by the JavaScript presentation until its native removal. */
export interface HostId {
  slot: number
  generation: number
}

/** A build-imported raster image or SVG with a JS-safe native renderer ID. */
export interface AssetRef {
  kind: 'image' | 'svg'
  id: number
}

/** A preferred size in logical pixels, a containing-block percentage, or intrinsic sizing. */
export type DimensionValue = number | 'auto' | `${number}%`

/** A minimum or maximum size constraint in logical pixels, percent, or auto. */
export type ConstraintValue = number | 'auto' | `${number}%`

/** One fixed, intrinsic, fractional or bounded native grid track. */
export type GridTrackSingle = number | 'auto' | `${number}%`
  | { fr: number }
  | { minmax: { min: number; max: number | { fr: number } } }

/** A grid track or one native repeat group. */
export type GridTrackValue = GridTrackSingle | {
  repeat: { count: number | 'autoFit' | 'autoFill'; tracks: readonly GridTrackSingle[] }
}

/** Ordered tracks for a row or column grid axis. */
export type GridTracksValue = readonly GridTrackValue[]

/** A native container condition and the layout values it applies when matched. */
export interface ContainerRuleValue {
  scope: string
  when: {
    minWidth?: number
    maxWidth?: number
    minHeight?: number
    maxHeight?: number
    orientation?: 'landscape' | 'portrait'
  }
  style: {
    gridColumns?: GridTracksValue
    gridRows?: GridTracksValue
    width?: DimensionValue
    height?: DimensionValue
    gap?: number
    grow?: number
    shrink?: number
    alignItems?: 'start' | 'center' | 'end' | 'stretch'
    justifyContent?: 'start' | 'center' | 'end' | 'spaceBetween' | 'spaceAround' | 'spaceEvenly'
  }
}

type VerticalInsets = {
  top?: number
  bottom?: number
}

/** Physical or writing-direction edges; horizontal modes cannot be mixed. */
export type InsetsValue = number | (VerticalInsets & (
  { right?: number; left?: number; start?: never; end?: never }
  | { start?: number; end?: number; right?: never; left?: never }
))

/** Positioned edges; omitted object sides stay automatic rather than becoming zero. */
export type PositionInsetsValue = InsetsValue

/** Per-corner radii in logical pixels. */
export type RadiiValue = number | {
  topLeft?: number
  topRight?: number
  bottomRight?: number
  bottomLeft?: number
}

/** Stroke surrounding one native surface. */
export interface BorderValue {
  width: number | { top: number; right: number; bottom: number; left: number }
  color: string
}

/** One native shadow, with offsets, blur and spread in logical pixels. */
export interface ShadowValue {
  offsetX?: number
  offsetY?: number
  blur: number
  spread?: number
  color: string
  inset?: boolean
}

/** Paint-only transform; it does not reserve layout space. */
export interface TransformValue {
  translateX?: number
  translateY?: number
  scaleX?: number
  scaleY?: number
  rotation?: number
}

/** A schema value with the Rust `SchemaValue` variant made explicit on the wire. */
export interface WireValue {
  type: string
  value: unknown
}

/** One operation matching a variant of `argui_host::Operation`. */
export type Operation =
  | { kind: 'create'; id: HostId; nativeType: number }
  | { kind: 'setProperty'; id: HostId; property: number; value: WireValue | null }
  | { kind: 'setListener'; id: HostId; event: number; callback: number | null }
  | { kind: 'insert'; parent: HostId; child: HostId; before: HostId | null }
  | { kind: 'remove'; id: HostId }
  | { kind: 'setRoot'; id: HostId | null }

/** Property metadata exported from the Rust `SchemaRegistry`. */
export interface NativeProperty {
  id: number
  name: string
  valueType: string
  readOnly: boolean
  required?: boolean
  default?: string | null
  allowedValues?: readonly string[]
}

/** Event metadata exported from the Rust `SchemaRegistry`. */
export interface NativeEvent {
  id: number
  name: string
  eventType?: string
  payload?: string | null
}

/** Native primitive metadata exported from the Rust `SchemaRegistry`. */
export interface NativeType {
  id: number
  name: string
  properties: NativeProperty[]
  events: NativeEvent[]
}

/** The complete runtime contract and its Rust ABI fingerprint. */
export interface NativeContract {
  abiHash: string
  natives: NativeType[]
}

/** Event delivery from the Rust host after stale generations were rejected. */
export interface NativeDelivery {
  node: HostId
  callback: number
  payload?: unknown
}

/** The QuickJS binding to `argui_host::Host`. */
export interface NativeBridge {
  contract(): NativeContract
  commit(operations: readonly Operation[]): void
  subscribe(deliver: (event: NativeDelivery) => void): () => void
  /** Application-owned theme session backed by `argui-theme`. */
  theme?: ThemeBridge
  /** Starts an application request separately from UI transactions. */
  request?(request: ServiceRequest): void
  /** Marks a pending application request as cancelled. */
  cancelRequest?(window: string, requestId: number): void
  /** Subscribes to application responses for this JavaScript session. */
  subscribeResponses?(deliver: (response: ServiceResponse) => void): () => void
}

/** Values and changes exchanged with the native theme runtime. */
export type ThemeWireValue = string | number | boolean

/** A coherent resolved theme revision returned by the host. */
export interface ThemeWireSnapshot {
  revision: number
  variant: string | null
  resolvedVariant: string | null
  systemScheme: 'light' | 'dark'
  values: Record<string, ThemeWireValue>
  tokenRevisions: Record<string, number>
  change?: { tokens: string[]; impact: 'Semantics' | 'Composite' | 'Paint' | 'Scroll' | 'Layout' | null }
}

/** Theme entry points implemented by a native or Web host. */
export interface ThemeBridge {
  create(definition: unknown): { id: number; snapshot: ThemeWireSnapshot }
  update(id: number, patch: unknown): ThemeWireSnapshot
  subscribe(id: number, deliver: (snapshot: ThemeWireSnapshot) => void): () => void
  dispose(id: number): void
}

/** One service call owned by a native window and a JavaScript session. */
export interface ServiceRequest {
  requestId: number
  window: string
  service: string
  method: string
  payload: unknown
}

/** A terminal response. Cancellation and platform support are explicit. */
export type ServiceResponse =
  | { requestId: number; window: string; status: 'ok'; value: unknown }
  | { requestId: number; window: string; status: 'cancelled' }
  | { requestId: number; window: string; status: 'error' | 'unsupported'; message: string }
  | { requestId: 0; window: string; status: 'event'; value: ApplicationEvent }

/** Native tray, global shortcut, and registration-failure events. */
export type ApplicationEvent =
  | { type: 'menu'; id: string }
  | { type: 'shortcut'; id: string; state: 'pressed' | 'released' }
  | { type: 'systemScheme'; scheme: 'light' | 'dark' }
  | { type: 'shortcutError' | 'trayError'; message: string }
  | { type: 'window'; window: string; message: string }
  | { type: 'windowAppearanceError'; window: string; message: string }
