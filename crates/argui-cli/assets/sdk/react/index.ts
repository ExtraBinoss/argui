import { createContext, type ReactNode } from 'react'
export { VirtualList } from './vlist'
export type { VirtualListProps } from './vlist'
export { useTheme, useThemeSnapshot } from './theme'
import Reconciler from 'react-reconciler'
import { ConcurrentRoot, DefaultEventPriority } from 'react-reconciler/constants'
import type { NativeHost } from '@argui/host'
import { commitWork, detach, markDirty, nativeInstance, place, updateProps, workNode, type WorkNode, type WorkRoot } from './tree'

type Props = Record<string, unknown>
type Timeout = ReturnType<typeof setTimeout>
type Config = Reconciler.HostConfig<
  string, Props, WorkRoot, WorkNode, WorkNode, never, never, never,
  WorkNode, object, never, Timeout, -1, null
>

let updatePriority = DefaultEventPriority
const hostContext = {}

const config: Config = {
  supportsMutation: true,
  supportsPersistence: false,
  supportsHydration: false,
  isPrimaryRenderer: true,
  supportsMicrotasks: true,
  scheduleMicrotask: queueMicrotask,
  scheduleTimeout: setTimeout,
  cancelTimeout: clearTimeout,
  noTimeout: -1,
  getRootHostContext: () => hostContext,
  getChildHostContext: () => hostContext,
  getPublicInstance: (instance) => instance,
  createInstance: (type, props, root) => workNode(root, type, props),
  createTextInstance: (text, root) => workNode(root, 'Text', {}, text),
  appendInitialChild: place,
  finalizeInitialChildren: () => false,
  shouldSetTextContent: () => false,
  prepareForCommit: () => null,
  resetAfterCommit: commitWork,
  preparePortalMount: () => {},
  appendChild: place,
  appendChildToContainer: place,
  insertBefore: (parent, child, before) => place(parent, child, before),
  insertInContainerBefore: (root, child, before) => place(root, child, before),
  removeChild: detach,
  removeChildFromContainer: detach,
  clearContainer: (root) => {
    for (const child of [...root.children]) detach(root, child)
  },
  commitUpdate: (instance, _type, previous, next) => updateProps(instance, previous, next),
  commitTextUpdate: (instance, _previous, next) => {
    instance.text = next
    markDirty(instance)
  },
  hideInstance: (instance) => { instance.hidden = true; markDirty(instance) },
  unhideInstance: (instance) => { instance.hidden = false; markDirty(instance) },
  hideTextInstance: (instance) => { instance.hidden = true; markDirty(instance) },
  unhideTextInstance: (instance) => { instance.hidden = false; markDirty(instance) },
  getInstanceFromNode: () => null,
  beforeActiveInstanceBlur: () => {},
  afterActiveInstanceBlur: () => {},
  prepareScopeUpdate: () => {},
  getInstanceFromScope: () => null,
  detachDeletedInstance: () => {},
  NotPendingTransition: null,
  HostTransitionContext: createContext(null) as unknown as Config['HostTransitionContext'],
  setCurrentUpdatePriority: (priority) => { updatePriority = priority },
  getCurrentUpdatePriority: () => updatePriority,
  resolveUpdatePriority: () => updatePriority,
  resetFormInstance: () => {},
  requestPostPaintCallback: (callback) => { setTimeout(() => callback(Date.now()), 0) },
  shouldAttemptEagerTransition: () => false,
  trackSchedulerEvent: () => {},
  resolveEventType: () => null,
  resolveEventTimeStamp: () => Date.now(),
  maySuspendCommit: () => false,
  preloadInstance: () => true,
  startSuspendingCommit: () => {},
  suspendInstance: () => {},
  waitForCommitToBeReady: () => null,
}

const reconciler = Reconciler(config)

/** A React root backed by the same native host used by Solid Universal. */
export interface ReactRoot {
  /** Commits React content through one native transaction. */
  render(element: ReactNode): void
  /** Removes React content and releases the native root and listeners. */
  unmount(): void
  /** Returns the stable native root for inspection and layout reads. */
  nativeRoot(): ReturnType<NativeHost['createElement']>
}

/** Creates a React root with one stable native primitive owned by `host`. */
export function createRoot(host: NativeHost, nativeType = 'Container', rootProperties: Props = {}): ReactRoot {
  const native = host.createElement(nativeType)
  for (const [name, value] of Object.entries(rootProperties)) host.setProperty(native, name, value)
  host.setRoot(native)
  const work: WorkRoot = { host, native, children: [], dirty: false }
  let uncaughtError: Error | null = null
  const root = reconciler.createContainer(
    work, ConcurrentRoot, null, false, null, '',
    (error) => { uncaughtError = error },
    () => {},
    () => {},
    () => {},
  )
  let mounted = true
  return {
    render(element) {
      if (!mounted) throw new Error('React root is unmounted')
      uncaughtError = null
      reconciler.updateContainerSync(element, root)
      reconciler.flushSyncWork()
      if (uncaughtError) throw uncaughtError
    },
    unmount() {
      if (!mounted) return
      reconciler.updateContainerSync(null, root)
      reconciler.flushSyncWork()
      host.dispose()
      mounted = false
    },
    nativeRoot: () => native,
  }
}

export { nativeInstance }
export type { WorkNode }
