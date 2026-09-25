/** A native text replacement whose range uses UTF-8 byte offsets. */
export interface InputEdit {
  kind: 'edit'
  start: number
  end: number
  text: string
}

/** Reads a native edit payload, rejecting malformed ranges. */
export function inputEdit(payload: unknown): InputEdit | undefined {
  if (!payload || typeof payload !== 'object') return undefined
  const edit = payload as Record<string, unknown>
  if (edit.kind !== 'edit' || !Number.isSafeInteger(edit.start) || !Number.isSafeInteger(edit.end)
    || (edit.start as number) < 0 || (edit.end as number) < (edit.start as number)
    || typeof edit.text !== 'string') return undefined
  return edit as unknown as InputEdit
}

/** Converts a UTF-8 byte boundary into a JavaScript UTF-16 index. */
function utf16Index(value: string, offset: number): number | undefined {
  let bytes = 0
  for (let index = 0; index < value.length;) {
    if (bytes === offset) return index
    const codepoint = value.codePointAt(index)!
    bytes += codepoint <= 0x7f ? 1 : codepoint <= 0x7ff ? 2 : codepoint <= 0xffff ? 3 : 4
    index += codepoint > 0xffff ? 2 : 1
    if (bytes > offset) return undefined
  }
  return bytes === offset ? value.length : undefined
}

/** Applies a native UTF-8 edit to a JavaScript string, or rejects an invalid boundary. */
export function applyInputEdit(value: string, edit: InputEdit, ascii = false): string | undefined {
  const start = ascii ? edit.start : utf16Index(value, edit.start)
  const end = ascii ? edit.end : utf16Index(value, edit.end)
  if (start === undefined || end === undefined || end > value.length || start > end) return undefined
  return value.slice(0, start) + edit.text + value.slice(end)
}
