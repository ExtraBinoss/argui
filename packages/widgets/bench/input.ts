import { performance } from 'node:perf_hooks'
import { applyInputEdit } from '@argui/host'
import { InputEditController } from '../src/shared/input-edit'

function report(label: string, samples: number[]) {
  samples.sort((left, right) => left - right)
  console.log(`${label}: p50=${samples[50].toFixed(3)} ms p95=${samples[95].toFixed(3)} ms max=${samples[99].toFixed(3)} ms / 100 keys`)
}

function burst(label: string, seed: string, position: number) {
  const controller = new InputEditController(seed)
  const samples: number[] = []
  let latest = seed
  for (let key = 0; key < 100; key++) {
    const start = performance.now()
    const accepted = controller.apply({ kind: 'edit', start: position + key, end: position + key, text: 'x' }, seed, (value) => { latest = value })
    if (!accepted) throw new Error(`rejected key ${key}`)
    samples.push(performance.now() - start)
  }
  if (latest.length !== seed.length + 100) throw new Error('lost edits')
  report(label, samples)
  const acknowledged = controller.apply({ kind: 'edit', start: position + 100, end: position + 100, text: 'x' }, latest, (value) => { latest = value })
  if (!acknowledged || latest.length !== seed.length + 101) throw new Error('lost acknowledgement')
}

burst('1M ASCII append Web bridge', 'a'.repeat(1_000_000), 1_000_000)
burst('1M Unicode scalars middle Web bridge', 'مرحبا'.repeat(200_000), 1_000_000)

function controlledMiddleAck() {
  const seed = 'a'.repeat(1_000_000)
  const middle = seed.length / 2
  const controller = new InputEditController(seed)
  let latest = seed
  for (let key = 0; key < 2_000; key++) {
    const offset = middle + key
    if (!controller.apply({ kind: 'edit', start: offset, end: offset, text: 'x' }, seed, (value) => { latest = value })) {
      throw new Error(`rejected key ${key}`)
    }
  }
  const partial = seed.slice(0, middle) + 'x'.repeat(1_000) + seed.slice(middle)
  let candidate = seed
  const replayStart = performance.now()
  for (let key = 0; key < 1_000; key++) {
    const offset = middle + key
    candidate = applyInputEdit(candidate, { kind: 'edit', start: offset, end: offset, text: 'x' }, true)!
  }
  if (candidate !== partial) throw new Error('replay diverged')
  const replay = performance.now() - replayStart
  const ackStart = performance.now()
  const accepted = controller.apply({ kind: 'edit', start: middle + 2_000, end: middle + 2_000, text: 'x' }, partial, (value) => { latest = value })
  const ack = performance.now() - ackStart
  if (!accepted || latest.length !== seed.length + 2_001 || controller.pendingCount !== 1_001) {
    throw new Error('partial acknowledgement lost edits')
  }
  console.log(`1M ASCII middle controlled 2000-key partial ack: replay=${replay.toFixed(3)} ms indexed ack+edit=${ack.toFixed(3)} ms`)
}

controlledMiddleAck()

function controlledNoncontiguousAck(keys: number) {
  const seed = 'a'.repeat(1_000_000)
  const controller = new InputEditController(seed)
  let latest = seed
  let partial = ''
  for (let key = 0; key < keys; key++) {
    const offset = key % 2 === 0 ? 250_000 : 750_000
    if (!controller.apply({ kind: 'edit', start: offset, end: offset, text: 'x' }, seed, (value) => { latest = value })) {
      throw new Error(`rejected key ${key}`)
    }
    if (key === keys / 2 - 1) partial = latest
  }
  const start = performance.now()
  const accepted = controller.apply({ kind: 'edit', start: 250_000, end: 250_000, text: 'x' }, partial, (value) => { latest = value })
  const elapsed = performance.now() - start
  if (!accepted || latest.length !== seed.length + keys + 1) throw new Error('noncontiguous acknowledgement lost edits')
  console.log(`1M ASCII noncontiguous controlled ${keys}-key partial ack+edit=${elapsed.toFixed(3)} ms`)
}

controlledNoncontiguousAck(2_000)
controlledNoncontiguousAck(10_000)

function controlledSameLengthAck() {
  const seed = 'a'.repeat(1_000_000)
  const controller = new InputEditController(seed)
  let latest = seed
  let partial = ''
  for (let key = 0; key < 10_000; key++) {
    const offset = key % 2 === 0 ? 250_000 + Math.floor(key / 2) : 750_000 + Math.floor(key / 2)
    if (!controller.apply({ kind: 'edit', start: offset, end: offset + 1, text: 'x' }, seed, (value) => { latest = value })) {
      throw new Error(`rejected replacement ${key}`)
    }
    if (key === 4_999) partial = latest
  }
  const start = performance.now()
  const accepted = controller.apply({ kind: 'edit', start: 250_000, end: 250_001, text: 'y' }, partial, (value) => { latest = value })
  const elapsed = performance.now() - start
  if (!accepted || latest.length !== seed.length) throw new Error('same-length acknowledgement lost edits')
  console.log(`1M ASCII same-length controlled 10000-key partial ack+edit=${elapsed.toFixed(3)} ms`)
}

controlledSameLengthAck()
