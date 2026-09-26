/** Returns distinct, non-empty messages in the order they were provided. */
export function uniqueFieldErrorMessages(
  errors: readonly (string | { message?: string } | undefined)[] | undefined,
): string[] {
  const seen = new Set<string>()
  const messages: string[] = []
  for (const error of errors ?? []) {
    const message = typeof error === 'string' ? error : error?.message
    if (!message || seen.has(message)) continue
    seen.add(message)
    messages.push(message)
  }
  return messages
}

/** Resolves a positive OTP cell count and bounds pathological layout sizes. */
export function inputOtpLength(length: number | undefined): number {
  return Number.isFinite(length) && length! > 0
    ? Math.min(24, Math.max(1, Math.floor(length!)))
    : 6
}

/** Keeps only ASCII digits and limits an OTP to its configured cell count. */
export function normalizeInputOtpValue(value: string, length: number): string {
  return value.replace(/[^0-9]/g, '').slice(0, length)
}
