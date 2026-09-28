/** Keeps the 35%-duration outward leg near 14 logical pixels per second. */
export function selectMarqueePeriod(overflow: number): number {
  return Math.max(4000, Math.ceil(overflow / 14 / 0.35 * 1000))
}

/** Aligns the selected row with the trigger after native focus reveals it. */
export function selectMenuOffset(triggerHeight: number, rowHeight: number, rowIndex: number,
  viewportHeight: number, shadowInset: number, borderWidth: number): number {
  const rowCenter = 4 + 26 + Math.max(0, rowIndex) * (rowHeight + 2) + rowHeight / 2
  const revealedScroll = Math.max(0, rowCenter + rowHeight / 2 - viewportHeight)
  return -(triggerHeight / 2 + shadowInset + borderWidth + rowCenter - revealedScroll)
}
