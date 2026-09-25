/** Visual and live-announcement priority for a notification. */
export type ToastVariant = 'default' | 'success' | 'info' | 'warning' | 'error' | 'loading'

/** Action rendered as a native button inside a notification. */
export interface ToastAction {
  /** Short label announced and displayed on the action button. */
  label: string
  /** Work performed when the user activates the action. */
  onClick: () => void
}

/** Optional content and lifetime for one notification. */
export interface ToastOptions {
  /** Supporting text displayed below the title. */
  description?: string
  /** Optional native button that performs an action and dismisses the notice. */
  action?: ToastAction
  /** Lifetime in milliseconds; zero or a non-finite value keeps the notice until dismissed. */
  duration?: number
  /** Whether the notice can be dismissed by its close button. Defaults to true. */
  dismissible?: boolean
}

/** Immutable content consumed by the Toast renderer. */
export interface ToastRecord {
  /** Stable identifier used to update and dismiss this notice. */
  id: string
  /** Main notification text. */
  title: string
  /** Supporting text. */
  description?: string
  /** Tone that controls the icon and live announcement priority. */
  variant: ToastVariant
  /** Optional action callback and button label. */
  action?: ToastAction
  /** Whether to show the close button. */
  dismissible: boolean
}

interface ToastTimer {
  handle?: ReturnType<typeof setTimeout>
  startedAt: number
  remaining: number
  paused: boolean
}

/** A shared external store for notifications shown by either framework adapter. */
export class ToastStore {
  private sequence = 0
  private records: readonly ToastRecord[] = []
  private readonly listeners = new Set<() => void>()
  private readonly timers = new Map<string, ToastTimer>()

  /** Returns the stable current snapshot until the next store mutation. */
  getSnapshot = (): readonly ToastRecord[] => this.records

  /** Subscribes to store changes and returns a function that removes the subscription. */
  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  /** Adds a notice and returns its identifier for later dismissal. */
  push(title: string, options: ToastOptions = {}, variant: ToastVariant = 'default'): string {
    const id = `toast-${++this.sequence}`
    const record: ToastRecord = {
      id,
      title,
      description: options.description,
      variant,
      action: options.action,
      dismissible: options.dismissible ?? true,
    }
    this.records = [...this.records, record]
    this.publish()
    this.schedule(id, options.duration ?? 4000)
    return id
  }

  /** Dismisses one notice, or every notice when the identifier is omitted. */
  dismiss(id?: string): void {
    if (id === undefined) {
      if (!this.records.length) return
      for (const record of this.records) this.clearTimer(record.id)
      this.records = []
      this.publish()
      return
    }
    if (!this.records.some((record) => record.id === id)) return
    this.clearTimer(id)
    this.records = this.records.filter((record) => record.id !== id)
    this.publish()
  }

  /** Pauses automatic dismissal while a user points at a notice. */
  pause(id: string): void {
    const timer = this.timers.get(id)
    if (!timer || timer.paused || timer.handle === undefined) return
    clearTimeout(timer.handle)
    timer.remaining = Math.max(0, timer.remaining - (Date.now() - timer.startedAt))
    timer.paused = true
  }

  /** Resumes a previously paused notice timer. */
  resume(id: string): void {
    const timer = this.timers.get(id)
    if (!timer || !timer.paused) return
    this.startTimer(id, timer, timer.remaining)
  }

  /** Replaces a notice while preserving its identifier, then resets its lifetime. */
  update(id: string, patch: Partial<Omit<ToastRecord, 'id'>>, duration = 4000): void {
    let found = false
    this.records = this.records.map((record) => {
      if (record.id !== id) return record
      found = true
      return { ...record, ...patch }
    })
    if (!found) return
    this.clearTimer(id)
    this.publish()
    this.schedule(id, duration)
  }

  private schedule(id: string, duration: number): void {
    if (!Number.isFinite(duration) || duration <= 0) return
    const timer: ToastTimer = { startedAt: Date.now(), remaining: duration, paused: false }
    this.timers.set(id, timer)
    this.startTimer(id, timer, duration)
  }

  private startTimer(id: string, timer: ToastTimer, duration: number): void {
    timer.startedAt = Date.now()
    timer.paused = false
    timer.handle = setTimeout(() => {
      this.timers.delete(id)
      this.dismiss(id)
    }, duration)
  }

  private clearTimer(id: string): void {
    const timer = this.timers.get(id)
    if (!timer) return
    if (timer.handle !== undefined) clearTimeout(timer.handle)
    this.timers.delete(id)
  }

  private publish(): void {
    for (const listener of [...this.listeners]) listener()
  }
}

/** Messages used by the promise helper while one notification changes state. */
export interface ToastPromiseMessages<T> {
  /** Text shown until the promise settles. */
  loading: string
  /** Success text or a formatter that receives the resolved value. */
  success: string | ((value: T) => string)
  /** Error text or a formatter that receives the rejected value. */
  error: string | ((error: unknown) => string)
  /** Lifetime of the final success or error notice. Defaults to four seconds. */
  duration?: number
}

/** Callable notification API with shadcn Sonner-style tone and promise helpers. */
export interface ToastApi {
  /** Pushes a default notice and returns its identifier. */
  (title: string, options?: ToastOptions): string
  /** Pushes a success notice. */
  success(title: string, options?: ToastOptions): string
  /** Pushes an informational notice. */
  info(title: string, options?: ToastOptions): string
  /** Pushes a warning notice. */
  warning(title: string, options?: ToastOptions): string
  /** Pushes an error notice. */
  error(title: string, options?: ToastOptions): string
  /** Pushes a persistent loading notice. */
  loading(title: string, options?: ToastOptions): string
  /** Removes one notice, or all notices when no identifier is supplied. */
  dismiss(id?: string): void
  /** Converts a promise's loading notice to a timed success or error notice. */
  promise<T>(operation: PromiseLike<T> | (() => PromiseLike<T>), messages: ToastPromiseMessages<T>): Promise<T>
}

/** Creates an isolated notification store for applications that need a non-default stack. */
export function createToastStore(): ToastStore {
  return new ToastStore()
}

/** Default cross-framework store mounted by a Sonner component. */
export const notificationStore = new ToastStore()

/** Adds default-variant notifications to the shared store. */
const notify = ((title: string, options?: ToastOptions) => notificationStore.push(title, options)) as ToastApi
notify.success = (title, options = {}) => notificationStore.push(title, options, 'success')
notify.info = (title, options = {}) => notificationStore.push(title, options, 'info')
notify.warning = (title, options = {}) => notificationStore.push(title, options, 'warning')
notify.error = (title, options = {}) => notificationStore.push(title, options, 'error')
notify.loading = (title, options = {}) => notificationStore.push(title, { ...options, duration: options.duration ?? 0 }, 'loading')
notify.dismiss = (id) => notificationStore.dismiss(id)
notify.promise = async <T>(operation: PromiseLike<T> | (() => PromiseLike<T>), messages: ToastPromiseMessages<T>): Promise<T> => {
  const id = notificationStore.push(messages.loading, { duration: 0, dismissible: false }, 'loading')
  try {
    const value = await (typeof operation === 'function' ? operation() : operation)
    const title = typeof messages.success === 'function' ? messages.success(value) : messages.success
    notificationStore.update(id, { title, variant: 'success', dismissible: true }, messages.duration ?? 4000)
    return value
  } catch (error) {
    const title = typeof messages.error === 'function' ? messages.error(error) : messages.error
    notificationStore.update(id, { title, variant: 'error', dismissible: true }, messages.duration ?? 4000)
    throw error
  }
}

/** Default callable API shared by the React and Solid widget adapters. */
export const toast: ToastApi = notify
