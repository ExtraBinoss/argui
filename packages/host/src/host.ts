import type {
  HostId,
  AssetRef,
  NativeBridge,
  NativeContract,
  NativeDelivery,
  NativeEvent,
  NativeProperty,
  NativeType,
  Operation,
  WireValue,
} from './protocol'
import { applyInputEdit, inputEdit } from './text-edit'

interface TextValueStamp { length: number; first: number; second: number }
interface TextValueHistory { entries: { value?: string; stamp?: TextValueStamp }[]; bytes: number }

/** Keeps a bounded fingerprint for delayed controlled-value acknowledgements. */
function textValueStamp(value: string): TextValueStamp {
  let first = 2166136261
  let second = 0x9e3779b9
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index)
    first = Math.imul(first ^ code, 16777619)
    second = Math.imul(second ^ code, 2246822519)
  }
  return { length: value.length, first, second }
}

/** A presentation node used only for framework navigation and pending mutations. */
export interface NativeNode {
  readonly id: HostId
  readonly type: NativeType
  parent: NativeNode | null
  children: NativeNode[]
  values: Map<number, WireValue>
  listeners: Map<number, number>
}

/** The shared presentation graph and transaction queue for framework adapters. */
export class NativeHost {
  readonly contract: NativeContract
  private readonly types = new Map<string, NativeType>()
  private readonly properties = new Map<string, NativeProperty>()
  private readonly events = new Map<string, NativeEvent>()
  private readonly callbacks = new Map<number, { node: NativeNode, event: string, handler: (payload: unknown) => void }>()
  private readonly textAscii = new WeakMap<NativeNode, boolean>()
  private readonly textHistory = new WeakMap<NativeNode, TextValueHistory>()
  private readonly pending: Operation[] = []
  private readonly unsubscribe: () => void
  private nextSlot = 1
  private nextCallback = 1
  private scheduled = false
  private root: NativeNode | null = null

  /** Validates `expectedAbiHash` against `bridge` and subscribes to native events. */
  constructor(private readonly bridge: NativeBridge, expectedAbiHash: string) {
    this.contract = bridge.contract()
    if (this.contract.abiHash !== expectedAbiHash) {
      throw new Error(`Argui schema hash mismatch: application ${expectedAbiHash}, native ${this.contract.abiHash}`)
    }
    for (const type of this.contract.natives) this.types.set(type.name, type)
    this.unsubscribe = bridge.subscribe((event) => this.deliver(event))
  }

  /** Creates one native primitive by its canonical Rust schema name. */
  createElement(name: string): NativeNode {
    const schemaName = name[0]!.toUpperCase() + name.slice(1)
    const type = this.types.get(schemaName)
    if (!type) throw new Error(`Unknown native primitive: ${name}`)
    const node: NativeNode = {
      id: { slot: this.nextSlot++, generation: 1 }, type, parent: null, children: [],
      values: new Map(), listeners: new Map(),
    }
    this.enqueue({ kind: 'create', id: node.id, nativeType: type.id })
    if (type.name === 'TextInput') this.textAscii.set(node, true)
    return node
  }

  /** Creates a `Text` primitive containing `value`. */
  createTextNode(value: string): NativeNode {
    const node = this.createElement('Text')
    this.setProperty(node, 'text', value)
    return node
  }

  /** Replaces a `Text` primitive's content. */
  replaceText(node: NativeNode, value: string): void {
    this.setProperty(node, 'text', value)
  }

  /** Sets a schema property or event handler on `node`. */
  setProperty(node: NativeNode, name: string, value: unknown): void {
    if (name === 'ref') {
      if (typeof value === 'function') value(node)
      return
    }
    if (name.startsWith('on') && name.length > 2) {
      this.setEvent(node, camelToSnake(name.slice(2)), value)
      return
    }
    const key = `${node.type.id}:${name}`
    let property = this.properties.get(key)
    if (property === undefined) {
      property = node.type.properties.find((entry) => entry.name === camelToSnake(name))
      if (property) this.properties.set(key, property)
    }
    if (!property) throw new Error(`${node.type.name} has no property ${name}`)
    if (property.readOnly) throw new Error(`${node.type.name}.${name} is read-only`)
    const wire = value == null ? null : encodeValue(property, value)
    if (node.type.name === 'TextInput' && property.name === 'value') {
      const history = this.textHistory.get(node)
      if (history && wire?.type === 'String' && typeof wire.value === 'string') {
        const nextValue = wire.value
        let stamp: TextValueStamp | undefined
        const acknowledged = history.entries.findIndex((entry) => entry.value !== undefined
          ? entry.value === nextValue
          : entry.stamp!.length === nextValue.length
            && entry.stamp!.first === (stamp ??= textValueStamp(nextValue)).first
            && entry.stamp!.second === stamp.second)
        if (acknowledged >= 0) {
          for (const entry of history.entries.splice(0, acknowledged + 1)) {
            if (entry.value !== undefined) history.bytes -= entry.value.length * 2
          }
          return
        }
      }
      this.textHistory.delete(node)
    }
    if (wire && equalValue(node.values.get(property.id), wire)) return
    if (!wire && !node.values.has(property.id)) return
    if (node.type.name === 'TextInput' && property.name === 'value') {
      this.textAscii.set(node, !wire || typeof wire.value === 'string' && /^[\x00-\x7f]*$/.test(wire.value))
    }
    if (wire) node.values.set(property.id, wire)
    else node.values.delete(property.id)
    this.enqueue({ kind: 'setProperty', id: node.id, property: property.id, value: wire })
  }

  /** Inserts or moves `child` under `parent` before an optional sibling. */
  insertNode(parent: NativeNode, child: NativeNode, before?: NativeNode | null): void {
    if (before && before.parent !== parent) throw new Error('Before node is not a child of parent')
    if (child === parent || isAncestor(child, parent)) throw new Error('Native presentation cycle')
    if (before === child) return
    if (child.parent) {
      const oldIndex = child.parent.children.indexOf(child)
      child.parent.children.splice(oldIndex, 1)
    }
    const index = before ? parent.children.indexOf(before) : parent.children.length
    parent.children.splice(index, 0, child)
    child.parent = parent
    this.enqueue({ kind: 'insert', parent: parent.id, child: child.id, before: before?.id ?? null })
  }

  /** Removes `child` and all of its descendants from the native host. */
  removeNode(parent: NativeNode, child: NativeNode): void {
    if (child.parent !== parent) throw new Error('Node is not a child of parent')
    parent.children.splice(parent.children.indexOf(child), 1)
    child.parent = null
    this.forgetCallbacks(child)
    this.enqueue({ kind: 'remove', id: child.id })
  }

  /** Returns whether `node` is a native text node. */
  isTextNode(node: NativeNode): boolean { return node.type.name === 'Text' }

  /** Returns the framework parent of `node`, if attached. */
  getParentNode(node: NativeNode): NativeNode | undefined { return node.parent ?? undefined }

  /** Returns the first framework child of `node`, if present. */
  getFirstChild(node: NativeNode): NativeNode | undefined { return node.children[0] }

  /** Returns the next sibling of `node`, if present. */
  getNextSibling(node: NativeNode): NativeNode | undefined {
    if (!node.parent) return undefined
    return node.parent.children[node.parent.children.indexOf(node) + 1]
  }

  /** Makes `node` the visible native root and flushes the initial transaction. */
  setRoot(node: NativeNode | null): void {
    if (node?.parent) throw new Error('Native root must be detached')
    const old = this.root
    this.root = node
    this.enqueue({ kind: 'setRoot', id: node?.id ?? null })
    if (old && old !== node) {
      this.forgetCallbacks(old)
      this.enqueue({ kind: 'remove', id: old.id })
    }
    this.flush()
  }

  /** Commits all pending operations in one atomic Rust host transaction. */
  flush(): void {
    if (this.pending.length === 0) return
    const batch = this.pending.splice(0)
    try {
      this.bridge.commit(batch)
    } catch (error) {
      this.pending.unshift(...batch)
      throw error
    }
  }

  /** Removes the visible root, listeners, and bridge event subscription. */
  dispose(): void {
    if (this.root) this.setRoot(null)
    this.callbacks.clear()
    this.unsubscribe()
  }

  private setEvent(node: NativeNode, name: string, value: unknown): void {
    const key = `${node.type.id}:${name}`
    let event = this.events.get(key)
    if (event === undefined) {
      event = node.type.events.find((entry) => entry.name === name)
      if (event) this.events.set(key, event)
    }
    if (!event) throw new Error(`${node.type.name} has no event ${name}`)
    if (value != null && typeof value !== 'function') {
      throw new TypeError(`${name} listener must be a function`)
    }
    const old = node.listeners.get(event.id)
    if (value == null) {
      if (old === undefined) return
      this.callbacks.delete(old)
      node.listeners.delete(event.id)
      this.enqueue({ kind: 'setListener', id: node.id, event: event.id, callback: null })
      return
    }
    if (old !== undefined) {
      this.callbacks.set(old, { node, event: name, handler: value as (payload: unknown) => void })
      return
    }
    const callback = this.nextCallback++
    this.callbacks.set(callback, { node, event: name, handler: value as (payload: unknown) => void })
    node.listeners.set(event.id, callback)
    this.enqueue({ kind: 'setListener', id: node.id, event: event.id, callback })
  }

  private enqueue(operation: Operation): void {
    this.pending.push(operation)
    if (!this.scheduled) {
      this.scheduled = true
      queueMicrotask(() => {
        this.scheduled = false
        this.flush()
      })
    }
  }

  private deliver(event: NativeDelivery): void {
    const listener = this.callbacks.get(event.callback)
    if (!listener || listener.node.id.slot !== event.node.slot
      || listener.node.id.generation !== event.node.generation) return
    if (listener.event === 'edit') this.mirrorTextEdit(listener.node, event.payload)
    listener.handler(event.payload)
  }

  /** Mirrors a native edit into the property cache so an unchanged controlled value needs no echo. */
  private mirrorTextEdit(node: NativeNode, payload: unknown): void {
    if (node.type.name !== 'TextInput') return
    const edit = inputEdit(payload)
    if (!edit) return
    const property = node.type.properties.find((entry) => entry.name === 'value')
    if (!property) return
    const wire = node.values.get(property.id)
    const current = wire?.type === 'String' && typeof wire.value === 'string' ? wire.value : ''
    const next = applyInputEdit(current, edit, this.textAscii.get(node))
    if (next === undefined) return
    let history = this.textHistory.get(node)
    if (!history || history.entries.length === 0) {
      history = { entries: [{ value: current }], bytes: current.length * 2 }
      this.textHistory.set(node, history)
    }
    history.entries.push({ value: next })
    history.bytes += next.length * 2
    while (history.bytes > 8 * 1024 * 1024 && history.entries.length > 1) {
      const old = history.entries.find((entry) => entry.value !== undefined)
      if (!old) break
      old.stamp = textValueStamp(old.value!)
      history.bytes -= old.value!.length * 2
      delete old.value
    }
    if (history.entries.length > 128) {
      const dropped = history.entries.shift()!
      if (dropped.value !== undefined) history.bytes -= dropped.value.length * 2
    }
    node.values.set(property.id, { type: 'String', value: next })
    this.textAscii.set(node, this.textAscii.get(node) === true && /^[\x00-\x7f]*$/.test(edit.text))
  }

  private forgetCallbacks(node: NativeNode): void {
    for (const callback of node.listeners.values()) this.callbacks.delete(callback)
    for (const child of node.children) this.forgetCallbacks(child)
  }
}

/** Converts a property or event suffix from JSX spelling to Rust schema spelling. */
function camelToSnake(name: string): string {
  return name.replace(/[A-Z]/g, (letter, index) => `${index ? '_' : ''}${letter.toLowerCase()}`)
}

/** Returns whether `ancestor` contains `node` in the pending presentation graph. */
function isAncestor(ancestor: NativeNode, node: NativeNode): boolean {
  for (let current: NativeNode | null = node; current; current = current.parent) {
    if (current === ancestor) return true
  }
  return false
}

/** Encodes a schema value for the Rust bridge's tagged `SchemaValue` decoder. */
function encodeValue(property: NativeProperty, value: unknown): WireValue {
  switch (property.valueType) {
    case 'Bool':
      if (typeof value !== 'boolean') break
      return { type: 'Bool', value }
    case 'Int':
      if (typeof value !== 'number' || !Number.isSafeInteger(value)) break
      return { type: 'Int', value }
    case 'Float':
      if (typeof value !== 'number' || !Number.isFinite(value)) break
      return { type: property.valueType, value }
    case 'String':
    case 'Name':
    case 'Color':
    case 'Brush':
      if (typeof value !== 'string') break
      return { type: property.valueType, value }
    case 'Dimension':
      if (typeof value !== 'number' && typeof value !== 'string') break
      return { type: 'Dimension', value }
    case 'Insets':
    case 'Radii':
    case 'Transform':
      if (typeof value !== 'number' && typeof value !== 'object') break
      return { type: property.valueType, value }
    case 'Asset':
      if (typeof value !== 'object' || !value) break
      {
        const asset = value as AssetRef
        if ((asset.kind !== 'image' && asset.kind !== 'svg') || !Number.isSafeInteger(asset.id) || asset.id <= 0) break
        return { type: 'Asset', value: { kind: asset.kind, id: asset.id } }
      }
  }
  throw new TypeError(`Invalid ${property.valueType} value for ${property.name}`)
}

/** Compares wire values, including stable asset identity, to avoid redundant native mutations. */
function equalValue(left: WireValue | undefined, right: WireValue): boolean {
  if (left?.type !== right.type) return false
  if (right.type === 'Asset') {
    const previous = left.value as AssetRef
    const next = right.value as AssetRef
    return previous.kind === next.kind && previous.id === next.id
  }
  return left.value === right.value
}
