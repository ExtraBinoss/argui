import type { NativeHost, NativeNode } from '@argui/host'

/** A React work node, which is kept separate from the visible Rust tree. */
export interface WorkNode {
  name: string
  props: Record<string, unknown>
  text: string | null
  parent: WorkNode | WorkRoot | null
  children: WorkNode[]
  native: NativeNode | null
  appliedProps: Record<string, unknown>
  appliedText: string | null
  hidden: boolean
  dirty: boolean
  root: WorkRoot
}

/** The accepted React tree and its stable native Container root. */
export interface WorkRoot {
  host: NativeHost
  native: NativeNode
  children: WorkNode[]
  dirty: boolean
}

/** Creates a detached React work node without invoking the native bridge. */
export function workNode(root: WorkRoot, name: string, props: Record<string, unknown>, text: string | null = null): WorkNode {
  return {
    name, props, text, parent: null, children: [], native: null,
    appliedProps: {}, appliedText: null, hidden: false, dirty: true, root,
  }
}

/** Inserts or moves a work node within React's pending tree. */
export function place(parent: WorkNode | WorkRoot, child: WorkNode, before: WorkNode | null = null): void {
  if (child.root !== (isWorkNode(parent) ? parent.root : parent)) {
    throw new Error('Cannot move React nodes between native roots')
  }
  if (before && (before.parent !== parent || before === child)) {
    if (before === child) return
    throw new Error('React insertion anchor is not a child of its parent')
  }
  if (parent === child || (isWorkNode(parent) && contains(child, parent))) {
    throw new Error('React work tree cycle')
  }
  if (child.parent) {
    child.parent.children.splice(child.parent.children.indexOf(child), 1)
    markDirty(child.parent)
  }
  const index = before ? parent.children.indexOf(before) : parent.children.length
  parent.children.splice(index, 0, child)
  child.parent = parent
  markDirty(parent)
}

/** Detaches a work node; its native subtree is removed at the accepted commit. */
export function detach(parent: WorkNode | WorkRoot, child: WorkNode): void {
  if (child.parent !== parent) throw new Error('React removal parent mismatch')
  parent.children.splice(parent.children.indexOf(child), 1)
  child.parent = null
  markDirty(parent)
}

/** Applies committed React props, refreshing mounted callbacks without walking unchanged subtrees. */
export function updateProps(node: WorkNode, previous: Record<string, unknown>, next: Record<string, unknown>): void {
  node.props = next
  let changed = false
  for (const key of Object.keys(previous)) {
    if (key === 'children' || key === 'key' || key === 'ref') continue
    const oldValue = previous[key]
    const newValue = next[key]
    if (Object.is(oldValue, newValue)) continue
    if (key.startsWith('on') && key.length > 2 && node.native
      && typeof oldValue === 'function' && typeof newValue === 'function') {
      node.root.host.setProperty(node.native, key, newValue)
    } else {
      changed = true
    }
  }
  for (const key of Object.keys(next)) {
    if (key === 'children' || key === 'key' || key === 'ref') continue
    if (!(key in previous)) changed = true
  }
  if (changed || !node.native) markDirty(node)
}

/** Marks a changed node and its ancestors for the next native transaction. */
export function markDirty(node: WorkNode | WorkRoot): void {
  for (let current: WorkNode | WorkRoot | null = node; current; current = isWorkNode(current) ? current.parent : null) {
    current.dirty = true
  }
}

/** Applies one accepted React commit to the shared native host in one batch. */
export function commitWork(root: WorkRoot): void {
  if (!root.dirty) return
  const live = new Set<NativeNode>()
  for (const child of root.children) collectNative(child, live)
  syncChildren(root.host, root.native, root.children, live)
  root.dirty = false
  root.host.flush()
}

/** Returns whether a work node is mounted to the native host. */
export function nativeInstance(node: WorkNode): NativeNode | null { return node.native }

function isWorkNode(node: WorkNode | WorkRoot): node is WorkNode { return 'name' in node }

function contains(ancestor: WorkNode, node: WorkNode): boolean {
  for (let current: WorkNode | WorkRoot | null = node; current; current = isWorkNode(current) ? current.parent : null) {
    if (current === ancestor) return true
  }
  return false
}

function collectNative(node: WorkNode, live: Set<NativeNode>): void {
  if (node.native) live.add(node.native)
  for (const child of node.children) collectNative(child, live)
}

function syncNode(host: NativeHost, node: WorkNode, live: Set<NativeNode>): NativeNode {
  if (!node.native) {
    node.native = node.text === null ? host.createElement(node.name) : host.createTextNode(node.text)
    node.dirty = true
    if (node.text !== null) node.appliedText = node.text
  }
  if (!node.dirty) return node.native
  if (node.text !== null) {
    if (node.text !== node.appliedText) host.replaceText(node.native, node.text)
    node.appliedText = node.text
    if (node.hidden && node.appliedProps.visible !== false) host.setProperty(node.native, 'visible', false)
    if (!node.hidden && node.appliedProps.visible === false) host.setProperty(node.native, 'visible', null)
    node.appliedProps = node.hidden ? { visible: false } : {}
  } else {
    const desired = nativeProps(node)
    for (const key of Object.keys(node.appliedProps)) {
      if (!(key in desired)) host.setProperty(node.native, key, null)
    }
    for (const [key, value] of Object.entries(desired)) {
      if (node.appliedProps[key] !== value) host.setProperty(node.native, key, value)
    }
    node.appliedProps = desired
    syncChildren(host, node.native, node.children, live)
  }
  node.dirty = false
  return node.native
}

function syncChildren(host: NativeHost, parent: NativeNode, children: WorkNode[], live: Set<NativeNode>): void {
  const desired = children.map((child) => syncNode(host, child, live))
  for (const child of [...parent.children]) {
    if (!live.has(child) && !desired.includes(child)) host.removeNode(parent, child)
  }
  for (let index = desired.length - 1; index >= 0; index--) {
    const child = desired[index]!
    const before = desired[index + 1] ?? null
    if (child.parent !== parent || host.getNextSibling(child) !== (before ?? undefined)) {
      host.insertNode(parent, child, before)
    }
  }
}

function nativeProps(node: WorkNode): Record<string, unknown> {
  const result: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(node.props)) {
    if (key === 'nativeKey') result.key = value
    else if (key !== 'children' && key !== 'key' && key !== 'ref') result[key] = value
  }
  if (node.hidden) result.visible = false
  return result
}
