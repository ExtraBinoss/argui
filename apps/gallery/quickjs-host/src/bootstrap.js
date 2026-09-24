const schema = JSON.parse(globalThis.__arguiContractJson)
globalThis.__arguiNativeEffects = []
let subscriber = null
let profileSubscriber = null
let nextTimer = 1
const timers = new Map()

globalThis.queueMicrotask = (callback) => Promise.resolve().then(callback)
const scheduleTimer = (callback, delay, repeat) => {
  const id = nextTimer++
  timers.set(id, { callback, delay: Math.max(1, Number(delay) || 0), due: 0, repeat })
  return id
}
globalThis.setInterval = (callback, delay) => scheduleTimer(callback, delay, true)
globalThis.setTimeout = (callback, delay) => scheduleTimer(callback, delay, false)
globalThis.clearInterval = (id) => { timers.delete(id) }
globalThis.clearTimeout = globalThis.clearInterval
globalThis.__arguiNextTimer = (now) => {
  let remaining = Infinity
  for (const timer of timers.values()) {
    if (timer.due === 0) timer.due = now + timer.delay
    remaining = Math.min(remaining, Math.max(0, timer.due - now))
  }
  return Number.isFinite(remaining) ? remaining : null
}
globalThis.__arguiTick = (now) => {
  for (const [id, timer] of timers) {
    if (timer.due === 0) timer.due = now + timer.delay
    if (now < timer.due) continue
    if (timer.repeat) timer.due = now + timer.delay
    else timers.delete(id)
    timer.callback()
  }
}
globalThis.__arguiDeliver = (json) => { subscriber?.(JSON.parse(json)) }
globalThis.__arguiDeliverProfile = (json) => { profileSubscriber?.(JSON.parse(json)) }
const i18nResponse = (json) => {
  const response = JSON.parse(json)
  if (response.error) throw new Error(response.error)
  return response
}
globalThis.__arguiBridge = {
  contract: () => schema,
  commit: (operations) => {
    const error = globalThis.__arguiSend(JSON.stringify(operations.map((op) => {
      switch (op.kind) {
        case 'create': return [0, op.id.slot, op.id.generation, op.nativeType]
        case 'setProperty': return [1, op.id.slot, op.id.generation, op.property, op.value?.type ?? null, op.value?.value ?? null]
        case 'setListener': return [2, op.id.slot, op.id.generation, op.event, op.callback]
        case 'insert': return [3, op.parent.slot, op.parent.generation, op.child.slot, op.child.generation, op.before?.slot ?? null, op.before?.generation ?? null]
        case 'remove': return [4, op.id.slot, op.id.generation]
        case 'setRoot': return [5, op.id?.slot ?? null, op.id?.generation ?? null]
        default: throw new Error(`Unknown native operation: ${op.kind}`)
      }
    })))
    if (error) throw new Error(error)
  },
  subscribe: (callback) => {
    subscriber = callback
    return () => { if (subscriber === callback) subscriber = null }
  },
  control: (request) => {
    const error = globalThis.__arguiControl(JSON.stringify(request))
    if (error) throw new Error(error)
  },
  subscribeProfile: (callback) => {
    profileSubscriber = callback
    return () => { if (profileSubscriber === callback) profileSubscriber = null }
  },
  i18n: {
    load: (config) => i18nResponse(globalThis.__arguiI18nLoad(JSON.stringify(config))),
    select: (locale) => i18nResponse(globalThis.__arguiI18nSelect(locale)),
    tr: (id, args = {}) => i18nResponse(globalThis.__arguiI18nTr(id, JSON.stringify(args))).value,
  },
}
