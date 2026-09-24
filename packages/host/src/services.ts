import type { ApplicationEvent, NativeBridge, ServiceRequest, ServiceResponse } from './protocol'

/** Error code returned by a native application service. */
export type ServiceErrorCode = 'cancelled' | 'unsupported' | 'error' | 'closed'

/** A native service failure with a machine-readable cause. */
export class ServiceError extends Error {
  /** Creates an error with the service's `code` and readable `message`. */
  constructor(readonly code: ServiceErrorCode, message: string) {
    super(message)
    this.name = 'ServiceError'
  }
}

/** A native file reference; contents stay outside the JavaScript bridge. */
export interface NativeFileReference {
  path: string
  name: string
}

/** Options accepted by the native file picker. */
export interface FileDialogOptions {
  title?: string
  fileName?: string
}

/** An application menu command or nested menu. */
export interface ApplicationMenuItem {
  id: string
  label: string
  enabled?: boolean
  children?: readonly ApplicationMenuItem[]
  /** Built-in window action; custom dispatches this item's ID to JavaScript. */
  action?: 'custom' | 'focus' | 'hide' | 'toggle' | 'quit'
}

/** A portable global accelerator owned by the application. */
export interface ApplicationShortcut {
  id: string
  accelerator: string
}

/** Live native window details; sizes use logical pixels. */
export interface NativeWindowInfo {
  window: string
  title: string
  width: number
  height: number
  visible: boolean | null
  decorations: boolean
  transparent: boolean
  backdrop: boolean
  backdropAvailable: boolean
}

type Pending = {
  resolve(value: unknown): void
  reject(error: ServiceError): void
  cleanup(): void
}

/** Per-window application services shared by React and Solid components. */
export class ApplicationServices {
  private nextRequestId = 1
  private pending = new Map<number, Pending>()
  private unsubscribe: (() => void) | undefined
  private closed = false
  private eventListeners = new Set<(event: ApplicationEvent) => void>()

  /** Subscribes to responses from `bridge` for native `window`. */
  constructor(private readonly bridge: NativeBridge, readonly window: string = 'main') {
    if (!window) throw new TypeError('Invalid native window ID')
    this.unsubscribe = bridge.subscribeResponses?.((response) => this.deliver(response))
  }

  /** Requests one typed service operation; `signal` cancels the pending result. */
  call<T>(service: string, method: string, payload: unknown = null, signal?: AbortSignal): Promise<T> {
    if (this.closed) return Promise.reject(new ServiceError('closed', 'Application service session closed'))
    if (!this.bridge.request || !this.unsubscribe) {
      return Promise.reject(new ServiceError('unsupported', 'Application services are unavailable'))
    }
    if (signal?.aborted) return Promise.reject(new ServiceError('cancelled', 'Request cancelled'))
    const requestId = this.nextRequestId++
    const request: ServiceRequest = { requestId, window: this.window, service, method, payload }
    return new Promise<T>((resolve, reject) => {
      const abort = () => {
        try { this.bridge.cancelRequest?.(this.window, requestId) } catch { /* Session may already be closing. */ }
        this.settle(requestId, { requestId, window: this.window, status: 'cancelled' })
      }
      signal?.addEventListener('abort', abort, { once: true })
      this.pending.set(requestId, {
        resolve: (value) => resolve(value as T), reject,
        cleanup: () => signal?.removeEventListener('abort', abort),
      })
      try { this.bridge.request!(request) }
      catch (error) {
        this.settle(requestId, {
          requestId, window: this.window, status: 'error', message: String(error),
        })
      }
    })
  }

  /** Reports whether a native service method is registered for this window. */
  supports(service: string, method: string): Promise<boolean> {
    return this.call<boolean>('host', 'supports', { service, method })
  }

  /** Reads UTF-8 text from the system clipboard. */
  readClipboardText(signal?: AbortSignal): Promise<string> {
    return this.call('clipboard', 'readText', null, signal)
  }

  /** Writes `text` to the system clipboard. */
  writeClipboardText(text: string, signal?: AbortSignal): Promise<void> {
    return this.call('clipboard', 'writeText', { text }, signal)
  }

  /** Opens one or more files and returns native references, never file bytes. */
  openFiles(options: FileDialogOptions & { multiple?: boolean } = {}, signal?: AbortSignal): Promise<NativeFileReference[]> {
    return this.call('files', options.multiple ? 'openMany' : 'open', options, signal)
  }

  /** Opens a save dialog and returns its selected native destination. */
  saveFile(options: FileDialogOptions = {}, signal?: AbortSignal): Promise<NativeFileReference> {
    return this.call('files', 'save', options, signal)
  }

  /** Replaces custom tray menu entries; Argui retains Show and Quit commands. */
  setMenu(items: readonly ApplicationMenuItem[], signal?: AbortSignal): Promise<void> {
    return this.call('menus', 'set', { items }, signal)
  }

  /** Replaces global accelerators where a shortcut backend is available. */
  setGlobalShortcuts(shortcuts: readonly ApplicationShortcut[], signal?: AbortSignal): Promise<void> {
    return this.call('shortcuts', 'set', { shortcuts }, signal)
  }

  /** Enables or removes the tray icon; disabling it restores quit-on-close. */
  setTrayEnabled(enabled: boolean, signal?: AbortSignal): Promise<void> {
    return this.call('tray', 'setEnabled', { enabled }, signal)
  }

  /** Chooses whether closing the main window hides it or quits the application. */
  setCloseBehavior(behavior: 'hide' | 'quit', signal?: AbortSignal): Promise<void> {
    return this.call('windows', 'setCloseBehavior', { behavior }, signal)
  }

  /** Shows and focuses the main window. */
  focusWindow(signal?: AbortSignal): Promise<void> {
    return this.call('windows', 'focus', null, signal)
  }

  /** Reads a native window's current logical size and requested appearance. */
  getWindowInfo(window: string = 'main', signal?: AbortSignal): Promise<NativeWindowInfo> {
    return this.call('windows', 'getInfo', { window }, signal)
  }

  /** Changes the native title of an existing window. */
  setWindowTitle(window: string, title: string, signal?: AbortSignal): Promise<void> {
    return this.call('windows', 'setTitle', { window, title }, signal)
  }

  /** Requests a new logical client size for an existing window. */
  setWindowSize(window: string, width: number, height: number, signal?: AbortSignal): Promise<void> {
    return this.call('windows', 'setSize', { window, width, height }, signal)
  }

  /** Enables or removes the native title bar and borders on an existing window. */
  setWindowDecorations(window: string, decorations: boolean, signal?: AbortSignal): Promise<void> {
    return this.call('windows', 'setDecorations', { window, decorations }, signal)
  }

  /** Subscribes to tray actions, global shortcuts, and native registration errors. */
  onEvent(listener: (event: ApplicationEvent) => void): () => void {
    if (this.closed) return () => {}
    this.eventListeners.add(listener)
    return () => { this.eventListeners.delete(listener) }
  }

  /** Cancels and rejects every pending call, then detaches the response listener. */
  dispose(): void {
    if (this.closed) return
    this.closed = true
    for (const requestId of this.pending.keys()) {
      try { this.bridge.cancelRequest?.(this.window, requestId) } catch { /* Session may already be closing. */ }
      this.settle(requestId, { requestId, window: this.window, status: 'error', message: 'Application service session closed' }, 'closed')
    }
    this.unsubscribe?.()
    this.eventListeners.clear()
  }

  private deliver(response: ServiceResponse): void {
    if (response.window !== this.window || this.closed) return
    if (response.status === 'event') {
      for (const listener of [...this.eventListeners]) listener(response.value)
      return
    }
    this.settle(response.requestId, response)
  }

  private settle(requestId: number, response: Exclude<ServiceResponse, { status: 'event' }>, override?: ServiceErrorCode): void {
    const pending = this.pending.get(requestId)
    if (!pending) return
    this.pending.delete(requestId)
    pending.cleanup()
    if (response.status === 'ok') pending.resolve(response.value)
    else pending.reject(new ServiceError(override ?? response.status, response.status === 'cancelled' ? 'Request cancelled' : response.message))
  }
}
