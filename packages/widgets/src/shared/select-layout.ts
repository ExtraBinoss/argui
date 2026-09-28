/** Aligns the initially selected option after native focus reveals a long menu. */
export function selectMenuOffset(triggerHeight: number, rowHeight: number, rowIndex: number,
  viewportHeight: number, shadowInset: number, borderWidth: number): number {
  const rowCenter = 4 + 26 + Math.max(0, rowIndex) * (rowHeight + 2) + rowHeight / 2
  const revealedScroll = Math.max(0, rowCenter + rowHeight / 2 - viewportHeight)
  return -(triggerHeight / 2 + shadowInset + borderWidth + rowCenter - revealedScroll)
}
