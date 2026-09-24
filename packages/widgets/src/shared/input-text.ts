/** Extracts edited text from the native input callback on desktop and Android. */
export function inputText(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (payload && typeof payload === 'object' && 'text' in payload
    && typeof payload.text === 'string') return payload.text
  return undefined
}
