import { applyInputEdit, inputEdit } from '@argui/host'

export { applyInputEdit, inputEdit } from '@argui/host'

interface ValueStamp { length: number, first: number, second: number }

/** Hashes a string only when a delayed acknowledgement outlives the small full-value cache. */
function valueStamp(value: string): ValueStamp {
  let first = 2166136261
  let second = 0x9e3779b9
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index)
    first = Math.imul(first ^ code, 16777619)
    second = Math.imul(second ^ code, 2246822519)
  }
  return { length: value.length, first, second }
}

/** Compares compact acknowledgements without retaining their complete strings. */
function sameStamp(left: ValueStamp, right: ValueStamp): boolean {
  return left.length === right.length && left.first === right.first && left.second === right.second
}

/** Keeps native deltas ordered across asynchronous controlled-property acknowledgements. */
export class InputEditController {
  private shadow: string
  private authored: string
  private ascii: boolean
  private readonly pending: { value?: string, stamp?: ValueStamp }[] = []
  private pendingBytes = 0

  /** Starts a controlled editor with its current authored value. */
  constructor(value: string) {
    this.shadow = value
    this.authored = value
    this.ascii = /^[\x00-\x7f]*$/.test(value)
  }

  /** Applies one native edit and calls `onChange` with the complete updated value. */
  apply(payload: unknown, authoredValue: string, onChange: (value: string) => void): boolean {
    if (authoredValue !== this.authored) {
      this.authored = authoredValue
      let authoredStamp: ValueStamp | undefined
      const acknowledged = this.pending.findIndex((pending) => pending.value !== undefined
        ? pending.value === authoredValue
        : sameStamp(pending.stamp!, authoredStamp ??= valueStamp(authoredValue)))
      if (acknowledged >= 0) {
        for (const pending of this.pending.splice(0, acknowledged + 1)) {
          if (pending.value !== undefined) this.pendingBytes -= pending.value.length * 2
        }
      }
      else {
        this.pending.length = 0
        this.pendingBytes = 0
        this.shadow = authoredValue
        this.ascii = /^[\x00-\x7f]*$/.test(authoredValue)
      }
    }
    const edit = inputEdit(payload)
    if (!edit) return false
    const next = applyInputEdit(this.shadow, edit, this.ascii)
    if (next === undefined) return false
    this.shadow = next
    this.ascii = this.ascii && /^[\x00-\x7f]*$/.test(edit.text)
    this.pending.push({ value: next })
    this.pendingBytes += next.length * 2
    while (this.pendingBytes > 8 * 1024 * 1024 && this.pending.length > 1) {
      const oldest = this.pending.slice(0, -1).find((pending) => pending.value !== undefined)
      if (!oldest) break
      oldest.stamp = valueStamp(oldest.value!)
      this.pendingBytes -= oldest.value!.length * 2
      delete oldest.value
    }
    if (this.pending.length > 128) {
      const dropped = this.pending.shift()!
      if (dropped.value !== undefined) this.pendingBytes -= dropped.value.length * 2
    }
    onChange(next)
    return true
  }
}
