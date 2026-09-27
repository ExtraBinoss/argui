/** Updates gallery search from a key typed while its background owns focus. */
export function searchFromBackgroundKey(current: string, event: {
  key: string
  text: string | null
  state: 'pressed' | 'released'
  control: boolean
  alt: boolean
  super: boolean
}): string {
  if (event.state !== 'pressed' || event.control || event.alt || event.super) return current
  if (event.key === 'Escape') return ''
  if (event.key === 'Backspace') return [...current].slice(0, -1).join('')
  const text = event.text ?? (event.key.length === 1 ? event.key : '')
  return text && !/[\u0000-\u001f\u007f]/.test(text) ? current + text : current
}
