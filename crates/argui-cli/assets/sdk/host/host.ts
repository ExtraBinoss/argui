import type {
  HostId,
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
import { encodeValue, equalValue } from './wire-values'

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
  id: HostId
  readonly type: NativeType
  parent: NativeNode | null
  children: NativeNode[]
  values: Map<number, WireValue>
  listeners: Map<number, number>
  inlineText?: boolean
}

/** Stable public reference shared by Solid and React, invalid after native removal. */
export interface NativeHandle {
  readonly id: string | null
  readonly hostId: HostId | null
  readonly mounted: boolean
}

/** The shared presentation graph and transaction queue for framework adapters. */
export class NativeHost {
  readonly contract: NativeContract
  private readonly types = new Map<string, NativeType>()
  private readonly properties = new Map<string, NativeProperty>()
  private readonly events = new Map<string, NativeEvent>()
  private readonly callbacks = new Map<number, { node: NativeNode, event: string, handler: (payload: unknown) => void }>()
  private readonly handlers = new WeakMap<NativeNode, Map<number, { event: string, handler: (payload: unknown) => void }>>()
  private readonly removed = new WeakSet<NativeNode>()
  private readonly textAscii = new WeakMap<NativeNode, boolean>()
  private readonly textHistory = new WeakMap<NativeNode, TextValueHistory>()
  private readonly identities = new Map<string, NativeNode>()
  private readonly autoIdentities = new WeakMap<NativeNode, string>()
  private readonly handles = new WeakMap<NativeNode, NativeHandle>()
  private readonly composedText = new WeakSet<NativeNode>()
  private settingInlineText = false
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
    const publicName = /^[a-z][a-zA-Z0-9]*$/
    for (const type of this.contract.natives) {
      for (const property of type.properties) {
        if (!publicName.test(property.name) || property.allowedValues?.some((value) => !publicName.test(value))) {
          throw new Error(`Invalid public schema property ${type.name}.${property.name}`)
        }
      }
      for (const event of type.events) {
        if (!publicName.test(event.name) || event.eventType && !publicName.test(event.eventType)) {
          throw new Error(`Invalid public schema event ${type.name}.${event.name}`)
        }
      }
      this.types.set(type.name, type)
    }
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
    if (type.properties.some((property) => property.name === 'id')) {
      const generated = `argui-${node.id.slot}-${node.id.generation}`
      this.autoIdentities.set(node, generated)
      this.setProperty(node, 'id', generated)
    }
    if (type.name === 'TextInput') this.textAscii.set(node, true)
    return node
  }

  /** Creates a `Text` primitive containing `value`. */
  createTextNode(value: string): NativeNode {
    const node = this.createElement('Text')
    this.setProperty(node, 'text', value)
    return node
  }

  /** Creates presentation text that can be folded into a parent `<text>` value. */
  createInlineTextNode(value: string): NativeNode {
    const type = this.types.get('Text')
    if (!type) throw new Error('Unknown native primitive: Text')
    const property = type.properties.find((entry) => entry.name === 'text')
    if (!property) throw new Error('Text primitive has no text property')
    return {
      id: { slot: this.nextSlot++, generation: 1 }, type, parent: null, children: [],
      values: new Map([[property.id, { type: 'String', value }]]), listeners: new Map(), inlineText: true,
    }
  }

  /** Replaces a `Text` primitive's content. */
  replaceText(node: NativeNode, value: string): void {
    if (node.inlineText) {
      const property = node.type.properties.find((entry) => entry.name === 'text')!
      node.values.set(property.id, { type: 'String', value })
      if (node.parent?.type.name === 'Text') this.updateInlineText(node.parent)
      return
    }
    this.setProperty(node, 'text', value)
  }

  /** Sets a schema property or event handler on `node`. */
  setProperty(node: NativeNode, name: string, value: unknown): void {
    if (name === 'id' && value == null) {
      value = `argui-${node.id.slot}-${node.id.generation}`
      this.autoIdentities.set(node, value as string)
    }
    if (node.type.name === 'Text' && name === 'text'
      && this.composedText.has(node) && !this.settingInlineText) {
      throw new Error('<text> accepts either children or text, not both')
    }
    if (name === 'ref') {
      if (typeof value === 'function') value(this.handle(node))
      return
    }
    if (name.startsWith('on') && name.length > 2) {
      this.setEvent(node, name[2]!.toLowerCase() + name.slice(3), value)
      return
    }
    const key = `${node.type.id}:${name}`
    let property = this.properties.get(key)
    if (property === undefined) {
      property = node.type.properties.find((entry) => entry.name === name)
      if (property) this.properties.set(key, property)
    }
    if (!property) throw new Error(`${node.type.name} has no property ${name}`)
    if (property.readOnly) throw new Error(`${node.type.name}.${name} is read-only`)
    const wire = value == null ? null : encodeValue(property, value)
    if (name === 'id') {
      if (wire?.type !== 'String' || !wire.value) throw new TypeError('Native id must be a non-empty string')
      const id = wire.value as string
      if (id !== this.autoIdentities.get(node)) this.autoIdentities.delete(node)
      const owner = this.identities.get(id)
      if (owner && owner !== node) throw new Error(`Duplicate native id: ${id}`)
      const previous = node.values.get(property.id)?.value
      if (typeof previous === 'string' && previous !== id) this.identities.delete(previous)
      this.identities.set(id, node)
    }
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
    if (!this.removed.has(node)) {
      let pending: Operation | undefined
      if (name === 'id') {
        for (let index = this.pending.length - 1; index >= 0; index--) {
          const operation = this.pending[index]!
          if (operation.kind === 'setProperty' && operation.id === node.id && operation.property === property.id) {
            pending = operation
            break
          }
        }
      }
      if (pending?.kind === 'setProperty') pending.value = wire
      else this.enqueue({ kind: 'setProperty', id: node.id, property: property.id, value: wire })
    }
  }

  /** Inserts or moves `child` under `parent` before an optional sibling. */
  insertNode(parent: NativeNode, child: NativeNode, before?: NativeNode | null): void {
    if (before && before.parent !== parent) throw new Error('Before node is not a child of parent')
    if (child === parent || isAncestor(child, parent)) throw new Error('Native presentation cycle')
    if (before === child) return
    if (child.parent) {
      const previous = child.parent
      const oldIndex = previous.children.indexOf(child)
      previous.children.splice(oldIndex, 1)
      if (previous.type.name === 'Text' && child.inlineText) this.updateInlineText(previous)
    }
    const index = before ? parent.children.indexOf(before) : parent.children.length
    parent.children.splice(index, 0, child)
    child.parent = parent
    if (parent.type.name === 'Text' && child.inlineText) {
      this.updateInlineText(parent)
      return
    }
    if (child.inlineText) this.materializeInlineText(child)
    const parentWasRemoved = this.removed.has(parent)
    this.revive(parent)
    this.revive(child)
    if (!parentWasRemoved) {
      this.enqueue({ kind: 'insert', parent: parent.id, child: child.id, before: before?.id ?? null })
    }
  }

  /** Removes `child` and all of its descendants from the native host. */
  removeNode(parent: NativeNode, child: NativeNode): void {
    if (child.parent !== parent) throw new Error('Node is not a child of parent')
    parent.children.splice(parent.children.indexOf(child), 1)
    child.parent = null
    if (parent.type.name === 'Text' && child.inlineText) {
      this.updateInlineText(parent)
      return
    }
    if (!this.removed.has(child)) {
      this.retire(child)
      this.enqueue({ kind: 'remove', id: child.id })
    }
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

  /** Returns a stable public reference for a node or a deferred framework instance. */
  handle(node: NativeNode | (() => NativeNode | null)): NativeHandle {
    if (typeof node !== 'function') {
      const existing = this.handles.get(node)
      if (existing) return existing
      const created = this.makeHandle(() => node)
      this.handles.set(node, created)
      return created
    }
    return this.makeHandle(node)
  }

  private makeHandle(resolve: () => NativeNode | null): NativeHandle {
    const host = this
    return {
      get id() {
        const node = resolve()
        const property = node?.type.properties.find((entry) => entry.name === 'id')
        const value = property && node?.values.get(property.id)?.value
        return typeof value === 'string' ? value : null
      },
      get hostId() { const node = resolve(); return node && host.isMounted(node) ? node.id : null },
      get mounted() { const node = resolve(); return !!node && host.isMounted(node) },
    }
  }

  private isMounted(node: NativeNode): boolean {
    if (node.inlineText || this.removed.has(node)) return false
    let current = node
    while (current.parent) current = current.parent
    return current === this.root
  }

  /** Makes `node` the visible native root and flushes the initial transaction. */
  setRoot(node: NativeNode | null): void {
    if (node?.parent) throw new Error('Native root must be detached')
    const old = this.root
    if (node) this.revive(node)
    this.root = node
    this.enqueue({ kind: 'setRoot', id: node?.id ?? null })
    if (old && old !== node) {
      this.retire(old)
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
    this.identities.clear()
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
      this.handlers.get(node)?.delete(event.id)
      node.listeners.delete(event.id)
      if (!this.removed.has(node)) this.enqueue({ kind: 'setListener', id: node.id, event: event.id, callback: null })
      return
    }
    let handlers = this.handlers.get(node)
    if (!handlers) {
      handlers = new Map()
      this.handlers.set(node, handlers)
    }
    handlers.set(event.id, { event: name, handler: value as (payload: unknown) => void })
    if (old !== undefined) {
      if (!this.removed.has(node)) this.callbacks.set(old, { node, event: name, handler: value as (payload: unknown) => void })
      return
    }
    const callback = this.nextCallback++
    node.listeners.set(event.id, callback)
    if (!this.removed.has(node)) {
      this.callbacks.set(callback, { node, event: name, handler: value as (payload: unknown) => void })
      this.enqueue({ kind: 'setListener', id: node.id, event: event.id, callback })
    }
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

  /** Retires a removed subtree and its callbacks until it is mounted again. */
  private retire(node: NativeNode): void {
    this.removed.add(node)
    const identity = node.type.properties.find((property) => property.name === 'id')
    if (identity) {
      const previousAuto = this.autoIdentities.get(node)
      if (previousAuto) {
        node.values.set(identity.id, { type: 'String', value: `argui-${node.id.slot}-${node.id.generation}` })
        this.autoIdentities.set(node, `argui-${node.id.slot}-${node.id.generation}`)
      }
      const value = node.values.get(identity.id)?.value
      if (typeof value === 'string') this.identities.delete(value)
    }
    for (const callback of node.listeners.values()) this.callbacks.delete(callback)
    for (const child of node.children) this.retire(child)
  }

  /** Recreates a previously removed subtree with fresh native identities. */
  private revive(node: NativeNode): void {
    if (!this.removed.delete(node)) return
    node.id = { slot: this.nextSlot++, generation: 1 }
    this.enqueue({ kind: 'create', id: node.id, nativeType: node.type.id })
    const identity = node.type.properties.find((property) => property.name === 'id')
    if (identity) {
      const value = node.values.get(identity.id)?.value
      if (typeof value === 'string') {
        const owner = this.identities.get(value)
        if (owner && owner !== node) throw new Error(`Duplicate native id: ${value}`)
        this.identities.set(value, node)
      }
    }
    for (const [property, value] of node.values) {
      this.enqueue({ kind: 'setProperty', id: node.id, property, value })
    }
    for (const [event, callback] of node.listeners) {
      const saved = this.handlers.get(node)?.get(event)
      if (!saved) continue
      this.callbacks.set(callback, { node, ...saved })
      this.enqueue({ kind: 'setListener', id: node.id, event, callback })
    }
    for (const child of node.children) {
      if (child.inlineText) continue
      this.revive(child)
      this.enqueue({ kind: 'insert', parent: node.id, child: child.id, before: null })
    }
  }

  private updateInlineText(parent: NativeNode): void {
    const property = parent.type.properties.find((entry) => entry.name === 'text')!
    if (!this.composedText.has(parent) && parent.values.has(property.id) && parent.children.length > 0) {
      throw new Error('<text> accepts either children or text, not both')
    }
    const content = parent.children.map((child) => {
      if (!child.inlineText) throw new TypeError('<text> children must resolve to text')
      return child.values.get(property.id)?.value ?? ''
    }).join('')
    this.settingInlineText = true
    try { this.setProperty(parent, 'text', content) } finally { this.settingInlineText = false }
    if (parent.children.length) this.composedText.add(parent)
    else this.composedText.delete(parent)
  }

  private materializeInlineText(node: NativeNode): void {
    node.inlineText = false
    this.enqueue({ kind: 'create', id: node.id, nativeType: node.type.id })
    for (const [property, value] of node.values) {
      this.enqueue({ kind: 'setProperty', id: node.id, property, value })
    }
    const identity = node.type.properties.find((entry) => entry.name === 'id')
    if (identity) {
      const generated = `argui-${node.id.slot}-${node.id.generation}`
      this.autoIdentities.set(node, generated)
      this.setProperty(node, 'id', generated)
    }
  }
}

/** Returns whether `ancestor` contains `node` in the pending presentation graph. */
function isAncestor(ancestor: NativeNode, node: NativeNode): boolean {
  for (let current: NativeNode | null = node; current; current = current.parent) {
    if (current === ancestor) return true
  }
  return false
}
