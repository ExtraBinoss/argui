const schema = JSON.parse(globalThis.__arguiContractJson)
let subscriber = null
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
globalThis.__arguiBridge = {
  contract: () => schema,
  commit: (operations) => {
    const error = globalThis.__arguiSend(JSON.stringify(operations))
    if (error) throw new Error(error)
  },
  subscribe: (callback) => {
    subscriber = callback
    return () => { if (subscriber === callback) subscriber = null }
  },
}
