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
}

/** Event metadata exported from the Rust `SchemaRegistry`. */
export interface NativeEvent {
  id: number
  name: string
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
}
