import type { NativeBridge } from '@argui/host'

type Request = (method: string, target: string, args: string) => number
type Resolver = { resolve: (value: unknown) => void; reject: (error: Error) => void }

const native = globalThis as typeof globalThis & {
  __arguiRequest?: Request
  __arguiResolve?: (id: number, error: string, result: string) => void
  __arguiTest?: ArguiTest
  __arguiTestState?: { done: boolean; error: string }
  __arguiStartTest?: () => void
}
const pending = new Map<number, Resolver>()

function request(method: string, target = '', args: unknown = {}): Promise<unknown> {
  return new Promise((resolve, reject) => {
    if (!native.__arguiRequest) {
      reject(new Error('Argui automation is available only in `argui test`'))
      return
    }
    const id = native.__arguiRequest(method, target, JSON.stringify(args))
    pending.set(id, { resolve, reject })
  })
}

native.__arguiResolve = (id, error, result) => {
  const step = pending.get(id)
  if (!step) return
  pending.delete(id)
  if (error) step.reject(new Error(error))
  else step.resolve(result ? JSON.parse(result) : undefined)
}

export interface Viewport {
  width: number
  height: number
  scale?: number
}

/** Stable application window key and its windowless test viewport. */
export class TestWindow {
  constructor(readonly id: string) {}

  /** Reads the current logical viewport for this window. */
  async info(): Promise<Viewport> {
    return await request('window.info', this.id) as Viewport
  }

  /** Resizes the test window's logical viewport and recomputes its scene. */
  async resize(viewport: Viewport): Promise<void> {
    await request('window.resize', this.id, viewport)
  }

  /** Requests an OS window position; the windowless test host reports unsupported. */
  async move(x: number, y: number): Promise<void> {
    await request('window.move', this.id, { x, y })
  }
}

export interface ClickOptions {
  button?: 'left' | 'right'
}

export interface ScrollOptions {
  x: number
  y: number
  unit?: 'pixels' | 'lines'
}

export interface PerformanceBudget {
  p95FrameTimeMsBelow?: number
  maxFrameTimeMsBelow?: number
  framesOverBudgetAtMost?: number
}

export class Locator {
  constructor(private readonly target: string) {}

  /** Moves the pointer to the element center without activating it. */
  async move(): Promise<void> {
    await request('move', this.target, {})
  }

  /** Presses and releases a mouse button at the element center. */
  async click(options: ClickOptions = {}): Promise<void> {
    await request('click', this.target, options)
  }

  /** Scrolls at the element center; positive y moves content down. */
  async scroll(options: ScrollOptions): Promise<void> {
    await request('scroll', this.target, options)
  }

  /** Replaces a text input through Argui's editing path. */
  async fill(value: string): Promise<void> {
    await request('fill', this.target, { value })
  }

  /** Drags the pointer from this element to `target`. */
  async dragTo(target: Locator): Promise<void> {
    await request('drag', this.target, { to: target.target })
  }
}

export interface TestUi {
  /** Addresses the stable window key; windowless tests mount `main`. */
  window(id: string): TestWindow
  getById(id: string): Locator
  at(x: number, y: number): Locator
  keyboard: { press(key: string): Promise<void> }
  expectText(text: string): Promise<void>
  /** Waits while Argui timers, animations, and native commits keep advancing. */
  wait(ms: number): Promise<void>
  /** Sleeps for wall time while application timers and animations stay frozen. */
  sleep(ms: number): Promise<void>
  /** Freezes application timers and animations at the current frame. */
  pause(): Promise<void>
  /** Continues application time from the frozen frame. */
  resume(): Promise<void>
  screenshot(name: string): Promise<void>
  expectPerformance(budget: PerformanceBudget): Promise<void>
}

const ui: TestUi = {
  window: id => new TestWindow(id),
  getById: id => new Locator(id),
  at: (x, y) => new Locator(`${x},${y}`),
  keyboard: { press: async key => { await request('key', '', { key }) } },
  expectText: async text => { await request('expectText', '', { text }) },
  wait: async ms => { await request('wait', '', { ms }) },
  sleep: async ms => { await request('sleep', '', { ms }) },
  pause: async () => { await request('pause') },
  resume: async () => { await request('resume') },
  screenshot: async name => { await request('screenshot', '', { name }) },
  expectPerformance: async budget => { await request('performance', '', budget) },
}

export interface ArguiTest {
  app: (bridge: NativeBridge, expectedAbiHash: string) => () => void
  viewport?: Viewport
  run: (ui: TestUi) => Promise<void> | void
}

/** Defines a test that mounts the real application in one QuickJS session. */
export function defineArguiTest(test: ArguiTest): ArguiTest {
  native.__arguiTest = test
  return test
}

/** Starts the configured test after the native host has mounted. */
export function startArguiTest(): void {
  const test = native.__arguiTest
  if (!test) throw new Error('Test module must export defineArguiTest({ app, run })')
  native.__arguiTestState = { done: false, error: '' }
  Promise.resolve().then(() => test.run(ui)).then(
    () => { native.__arguiTestState = { done: true, error: '' } },
    error => { native.__arguiTestState = { done: true, error: String(error) } },
  )
}

native.__arguiStartTest = startArguiTest
