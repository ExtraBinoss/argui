/** Searchable item metadata shared by the native Command adapters. */
export interface CommandEntry {
  /** Stable unique item value and native accessibility key suffix. */
  value: string
  /** Visible and searchable item label. */
  label: string
  /** Whether activation and keyboard navigation should skip this item. */
  disabled: boolean
  /** Optional group name used to hide groups with no matching rows. */
  group?: string
}

/** Filters command entries by case-insensitive label or value substring. */
export function foundationJFilter<T extends CommandEntry>(entries: readonly T[], query: string): T[] {
  const needle = query.trim().toLocaleLowerCase()
  return entries.filter((entry) => !needle || entry.label.toLocaleLowerCase().includes(needle)
    || entry.value.toLocaleLowerCase().includes(needle))
}

/** Finds the next enabled command, wrapping around the matching entry list. */
export function foundationJNext(entries: readonly CommandEntry[], current: string | undefined, step: -1 | 1): string | undefined {
  const enabled = entries.filter((entry) => !entry.disabled)
  if (enabled.length === 0) return undefined
  const index = enabled.findIndex((entry) => entry.value === current)
  if (index < 0) return step === 1 ? enabled[0]?.value : enabled.at(-1)?.value
  return enabled[(index + step + enabled.length) % enabled.length]?.value
}
