import type { InputEdit } from '@argui/host'

interface Checkpoint { bytes: number; utf16: number }

function utf8Length(codepoint: number): number {
  return codepoint <= 0x7f ? 1 : codepoint <= 0x7ff ? 2 : codepoint <= 0xffff ? 3 : 4
}

function upperBound(checkpoints: Checkpoint[], bytes: number): number {
  let low = 0
  let high = checkpoints.length
  while (low < high) {
    const mid = (low + high) >>> 1
    if (checkpoints[mid].bytes <= bytes) low = mid + 1
    else high = mid
  }
  return low
}

/** Maps native UTF-8 edit offsets to JS UTF-16 positions using sparse checkpoints. */
export class InputOffsetIndex {
  private checkpoints: Checkpoint[] = [{ bytes: 0, utf16: 0 }]
  private bytes = 0

  constructor(value: string) {
    let bytes = 0
    let last = 0
    for (let utf16 = 0; utf16 < value.length;) {
      if (utf16 - last >= 1024) {
        this.checkpoints.push({ bytes, utf16 })
        last = utf16
      }
      const codepoint = value.codePointAt(utf16)!
      bytes += utf8Length(codepoint)
      utf16 += codepoint > 0xffff ? 2 : 1
    }
    this.bytes = bytes
  }

  /** Total UTF-8 bytes represented by the indexed value. */
  get byteLength(): number {
    return this.bytes
  }

  /** Number of retained checkpoints, for bounded-memory diagnostics. */
  get checkpointCount(): number {
    return this.checkpoints.length
  }

  /** Returns the UTF-16 boundary for a UTF-8 byte offset, or undefined inside a code point. */
  at(value: string, offset: number): number | undefined {
    const checkpoint = this.checkpoints[upperBound(this.checkpoints, offset) - 1]
    let bytes = checkpoint.bytes
    for (let utf16 = checkpoint.utf16; utf16 < value.length;) {
      if (bytes === offset) return utf16
      const codepoint = value.codePointAt(utf16)!
      bytes += utf8Length(codepoint)
      utf16 += codepoint > 0xffff ? 2 : 1
      if (bytes > offset) return undefined
    }
    return bytes === offset ? value.length : undefined
  }

  /** Moves checkpoints after an accepted edit without rescanning unchanged text. */
  update(edit: InputEdit, start: number, end: number): void {
    const first = upperBound(this.checkpoints, edit.start)
    const after = edit.end === edit.start ? first : upperBound(this.checkpoints, edit.end - 1)
    this.checkpoints.splice(first, after - first)
    let insertedBytes = 0
    let last = 0
    const inserted: Checkpoint[] = []
    for (let utf16 = 0; utf16 < edit.text.length;) {
      if (utf16 - last >= 1024) {
        inserted.push({ bytes: edit.start + insertedBytes, utf16: start + utf16 })
        last = utf16
      }
      const codepoint = edit.text.codePointAt(utf16)!
      insertedBytes += utf8Length(codepoint)
      utf16 += codepoint > 0xffff ? 2 : 1
    }
    const byteShift = insertedBytes - (edit.end - edit.start)
    this.bytes += byteShift
    const utf16Shift = edit.text.length - (end - start)
    for (let index = first; index < this.checkpoints.length; index++) {
      this.checkpoints[index].bytes += byteShift
      this.checkpoints[index].utf16 += utf16Shift
    }
    if (inserted.length) this.checkpoints.splice(first, 0, ...inserted)
    const caretBytes = edit.start + insertedBytes
    const caretUtf16 = start + edit.text.length
    const caretIndex = upperBound(this.checkpoints, caretBytes)
    if (this.checkpoints[caretIndex - 1]?.bytes === caretBytes) {
      this.checkpoints[caretIndex - 1].utf16 = caretUtf16
    } else {
      this.checkpoints.splice(caretIndex, 0, { bytes: caretBytes, utf16: caretUtf16 })
      if (caretIndex > 1 && caretBytes - this.checkpoints[caretIndex - 1].bytes < 1024) {
        this.checkpoints.splice(caretIndex - 1, 1)
      }
    }
  }
}
