import { createRenderer } from 'solid-js/universal'
import type { NativeHost, NativeNode } from '@argui/host'

let activeHost: NativeHost | null = null

/** Installs the native host used by Solid's universal renderer. */
export function useNativeHost(host: NativeHost): void {
  activeHost = host
}

/** Returns the configured native host or reports a missing bootstrap. */
export function nativeHost(): NativeHost {
  if (!activeHost) throw new Error('Argui Solid renderer has no native host')
  return activeHost
}

const renderer = createRenderer<NativeNode>({
  createElement: (name) => nativeHost().createElement(name),
  createTextNode: (value) => nativeHost().createTextNode(value),
  replaceText: (node, value) => nativeHost().replaceText(node, value),
  setProperty: (node, name, value) => nativeHost().setProperty(node, name, value),
  insertNode: (parent, node, anchor) => nativeHost().insertNode(parent, node, anchor),
  isTextNode: (node) => nativeHost().isTextNode(node),
  removeNode: (parent, node) => nativeHost().removeNode(parent, node),
  getParentNode: (node) => nativeHost().getParentNode(node),
  getFirstChild: (node) => nativeHost().getFirstChild(node),
  getNextSibling: (node) => nativeHost().getNextSibling(node),
})

export const {
  render, effect, memo, createComponent, insert, spread, setProp, mergeProps,
  createElement, createTextNode, insertNode, use,
} = renderer
export { For, Show, Switch, Match, createSignal, createMemo, onCleanup } from 'solid-js'
export { VirtualList } from './vlist'
export type { VirtualListProps } from './vlist'
