/** One plain or grouped option in an Argui selection popup. */
export type SelectOption = string | {
  value: string
  label?: string
  group?: string
  disabled?: boolean
}

/** Normalized option consumed by the native Solid and React adapters. */
export interface ResolvedSelectOption {
  value: string
  label: string
  group?: string
  disabled: boolean
}

/** Resolves concise string options and explicit option records uniformly. */
export function resolveSelectOptions(options: readonly SelectOption[]): ResolvedSelectOption[] {
  return options.map((option) => typeof option === 'string'
    ? { value: option, label: option, disabled: false }
    : { value: option.value, label: option.label ?? option.value,
      group: option.group, disabled: !!option.disabled })
}

/** Finds the next selectable index, wrapping while skipping disabled options. */
export function nextSelectIndex(options: readonly ResolvedSelectOption[], start: number, step: number): number {
  if (options.length === 0) return -1
  for (let offset = 1; offset <= options.length; offset++) {
    const index = ((start + step * offset) % options.length + options.length) % options.length
    if (!options[index]!.disabled) return index
  }
  return -1
}
