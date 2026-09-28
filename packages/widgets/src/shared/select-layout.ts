/** Keeps the 35%-duration outward leg near 14 logical pixels per second. */
export function selectMarqueePeriod(overflow: number): number {
  return Math.max(4000, Math.ceil(overflow / 14 / 0.35 * 1000))
}
