import { inputEdit, type InputEdit } from '@argui/host'
import { InputOffsetIndex } from './input-offset-index'

/** Applies ordered native text edits while controlled values acknowledge asynchronously. */
export class InputEditController {
  private shadow: string
  private authored: string
  private ascii: boolean
  private offsetIndex?: InputOffsetIndex
  private expectsAck: boolean
  private readonly pending: { value?: string; length: number; checksum: number; edit: InputEdit }[] = []
  private pendingBytes = 0
  private pendingInsert?: { start: number; text: string }
  private readonly checkpoints = new InputAckCheckpoints()
  private contentChecksum = 0
  private authoredChecksum = 0

  constructor(value: string, expectsAck = true) {
    this.shadow = value
    this.authored = value
    this.expectsAck = expectsAck
    this.ascii = /^[\x00-\x7f]*$/.test(value)
    if (!this.ascii) this.offsetIndex = new InputOffsetIndex(value)
  }

  /** Current native-shadow value used when a submit payload has no text. */
  get currentValue(): string {
    return this.shadow
  }

  /** Number of unacknowledged controlled edits retained for diagnostics. */
  get pendingCount(): number {
    return this.pending.length
  }

  apply(payload: unknown, authoredValue: string, onValueChange: (value: string) => void, expectsAck = this.expectsAck): boolean {
    if (expectsAck !== this.expectsAck) {
      this.expectsAck = expectsAck
      this.pending.length = 0
      this.pendingBytes = 0
      this.pendingInsert = undefined
      this.checkpoints.clear()
      this.contentChecksum = 0
      this.authoredChecksum = 0
      this.authored = authoredValue
      if (expectsAck) {
        this.shadow = authoredValue
        this.ascii = /^[\x00-\x7f]*$/.test(authoredValue)
        this.offsetIndex = this.ascii ? undefined : new InputOffsetIndex(authoredValue)
      }
    }
    if (expectsAck && authoredValue !== this.authored) {
      const previousAuthored = this.authored
      this.authored = authoredValue
      let acknowledged = -1
      let insertedUnits = 0
      const run = this.pendingInsert
      if (run && authoredValue.length > previousAuthored.length) {
        const added = authoredValue.length - previousAuthored.length
        if (added <= run.text.length
          && authoredValue.startsWith(previousAuthored.slice(0, run.start))
          && authoredValue.slice(run.start, run.start + added) === run.text.slice(0, added)
          && authoredValue.endsWith(previousAuthored.slice(run.start))) {
          acknowledged = this.pending.findIndex((pending) => pending.length === authoredValue.length)
          if (acknowledged >= 0) insertedUnits = added
        }
      }
      if (acknowledged < 0) acknowledged = this.pending.findIndex((pending) => pending.value === authoredValue)
      if (acknowledged < 0 && authoredValue === this.shadow) acknowledged = this.pending.length - 1
      if (acknowledged < 0 && this.pending.length) {
        const echoChecksum = (this.authoredChecksum + unitSum(authoredValue) - unitSum(previousAuthored)) >>> 0
        for (let index = 0; index < this.pending.length; index++) {
          const pending = this.pending[index]
          if (pending.length === authoredValue.length && pending.checksum === echoChecksum
            && piecesMatch(previousAuthored, authoredValue, this.pending, this.checkpoints.before(index + 1), index)) {
            acknowledged = index
            break
          }
        }
      }
      if (acknowledged >= 0) {
        this.authoredChecksum = this.pending[acknowledged].checksum
        for (const pending of this.pending.splice(0, acknowledged + 1)) {
          if (pending.value !== undefined) this.pendingBytes -= pending.value.length * 2
        }
        this.checkpoints.acknowledge(acknowledged + 1)
        if (run && insertedUnits > 0 && this.pending.length) {
          run.start += insertedUnits
          run.text = run.text.slice(insertedUnits)
        } else {
          this.pendingInsert = undefined
        }
        if (!this.pending.length) {
          this.checkpoints.clear()
          this.contentChecksum = 0
          this.authoredChecksum = 0
        }
      } else {
        this.pending.length = 0
        this.pendingBytes = 0
        this.pendingInsert = undefined
        this.checkpoints.clear()
        this.contentChecksum = 0
        this.authoredChecksum = 0
        this.shadow = authoredValue
        this.ascii = /^[\x00-\x7f]*$/.test(authoredValue)
        this.offsetIndex = this.ascii ? undefined : new InputOffsetIndex(authoredValue)
      }
    }
    const edit = inputEdit(payload)
    if (!edit) return false
    const start = this.ascii ? edit.start : this.offsetIndex!.at(this.shadow, edit.start)
    const end = this.ascii ? edit.end : this.offsetIndex!.at(this.shadow, edit.end)
    if (start === undefined || end === undefined || start > end || end > this.shadow.length) return false
    const removed = this.shadow.slice(start, end)
    const next = this.shadow.slice(0, start) + edit.text + this.shadow.slice(end)
    if (this.offsetIndex) this.offsetIndex.update(edit, start, end)
    this.shadow = next
    this.ascii = this.ascii && /^[\x00-\x7f]*$/.test(edit.text)
    if (!this.ascii && !this.offsetIndex) this.offsetIndex = new InputOffsetIndex(next)
    if (expectsAck) {
      this.contentChecksum = (this.contentChecksum + unitSum(edit.text) - unitSum(removed)) >>> 0
      if (!this.pending.length && start === end && edit.text.length) {
        this.pendingInsert = { start, text: edit.text }
      } else if (this.pendingInsert) {
        if (start === end && start === this.pendingInsert.start + this.pendingInsert.text.length && edit.text.length) {
          this.pendingInsert.text += edit.text
        } else {
          this.pendingInsert = undefined
        }
      }
      this.pending.push({ value: next, length: next.length, checksum: this.contentChecksum, edit })
      this.checkpoints.record(this.pending.length, next)
      this.pendingBytes += next.length * 2
      while (this.pendingBytes > 8 * 1024 * 1024 && this.pending.length > 1) {
        const oldest = this.pending.find((pending) => pending.value !== undefined)
        if (!oldest) break
        this.pendingBytes -= oldest.value!.length * 2
        delete oldest.value
      }
    }
    onValueChange(next)
    return true
  }
}

/** Extracts a submitted string from the native input callback. */
export function submittedText(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (payload && typeof payload === 'object' && 'text' in payload && typeof payload.text === 'string') {
    return payload.text
  }
  return undefined
}

/** Additive UTF-16 checksum used only to narrow candidates before exact comparison. */
function unitSum(value: string): number {
  let sum = 0
  for (let index = 0; index < value.length; index++) sum += value.charCodeAt(index)
  return sum >>> 0
}

/** Sparse full values keep the mixed-edit replay span short within 32 MiB. */
class InputAckCheckpoints {
  private values: { count: number; value: string }[] = []
  private bytes = 0
  private interval = 256
  private next = 256

  record(count: number, value: string): void {
    if (count !== this.next || value.length * 2 > 32 * 1024 * 1024) return
    this.values.push({ count, value })
    this.bytes += value.length * 2
    this.next = count + this.interval
    while (this.bytes > 32 * 1024 * 1024 && this.values.length > 1) {
      this.values = this.values.filter((checkpoint, index) => {
        if (index % 2 === 1) return true
        this.bytes -= checkpoint.value.length * 2
        return false
      })
      this.interval *= 2
      this.next = count + this.interval
    }
  }

  before(count: number): { count: number; value: string } | undefined {
    for (let index = this.values.length - 1; index >= 0; index--) {
      if (this.values[index].count <= count) return this.values[index]
    }
    return undefined
  }

  acknowledge(count: number): void {
    this.values = this.values.filter((checkpoint) => {
      if (checkpoint.count > count) {
        checkpoint.count -= count
        return true
      }
      this.bytes -= checkpoint.value.length * 2
      return false
    })
    this.next -= count
  }

  clear(): void {
    this.values = []
    this.bytes = 0
    this.interval = 256
    this.next = 256
  }
}

interface InputPiece {
  source: string
  startUnit: number
  endUnit: number
  startByte: number
  endByte: number
  ascii: boolean
  index?: InputOffsetIndex
}

/** Creates one piece from a complete edit or checkpoint value. */
function wholePiece(value: string): InputPiece {
  const ascii = /^[\x00-\x7f]*$/.test(value)
  const index = ascii ? undefined : new InputOffsetIndex(value)
  return {
    source: value,
    startUnit: 0,
    endUnit: value.length,
    startByte: 0,
    endByte: index?.byteLength ?? value.length,
    ascii,
    index,
  }
}

/** Splits the piece list at a UTF-8 boundary and returns its piece index. */
function splitPiece(pieces: InputPiece[], offset: number): number | undefined {
  let position = 0
  for (let index = 0; index < pieces.length; index++) {
    if (offset === position) return index
    const piece = pieces[index]
    const length = piece.endByte - piece.startByte
    if (offset < position + length) {
      const local = offset - position
      const absoluteByte = piece.startByte + local
      const units = piece.ascii ? piece.startUnit + local : piece.index!.at(piece.source, absoluteByte)
      if (units === undefined) return undefined
      pieces.splice(index, 1,
        { ...piece, endUnit: units, endByte: absoluteByte },
        { ...piece, startUnit: units, startByte: absoluteByte })
      return index + 1
    }
    position += length
  }
  return offset === position ? pieces.length : undefined
}

/** Verifies one pending value against the echo after applying local piece edits. */
function piecesMatch(
  base: string,
  echo: string,
  pending: { edit: InputEdit }[],
  checkpoint: { count: number; value: string } | undefined,
  target: number,
): boolean {
  const start = checkpoint?.count ?? 0
  const value = checkpoint?.value ?? base
  const pieces: InputPiece[] = [wholePiece(value)]
  for (let index = start; index <= target; index++) {
    const edit = pending[index].edit
    const first = splitPiece(pieces, edit.start)
    const last = splitPiece(pieces, edit.end)
    if (first === undefined || last === undefined || first > last) return false
    const replacement = edit.text ? [wholePiece(edit.text)] : []
    pieces.splice(first, last - first, ...replacement)
  }
  let position = 0
  for (const piece of pieces) {
    const text = piece.source.slice(piece.startUnit, piece.endUnit)
    if (!echo.startsWith(text, position)) return false
    position += text.length
  }
  return position === echo.length
}
