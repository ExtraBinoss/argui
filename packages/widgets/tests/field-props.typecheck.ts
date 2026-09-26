import type { InputFieldOptions, SelectOptions } from '../src/shared/types'

const change = (_value: string) => {}
const toggle = (_open: boolean) => {}

const autonomousInput = { accessibleName: 'Search', defaultValue: 'Argui', type: 'search' } satisfies InputFieldOptions
const controlledInput = { label: 'Name', value: 'Ada', onValueChange: change } satisfies InputFieldOptions
const readOnlyInput = { label: 'Reference', value: 'Ada', readOnly: true } satisfies InputFieldOptions
const inputWithHelp = { label: 'Code', description: 'Required for access', required: true } satisfies InputFieldOptions

// @ts-expect-error A controlled input must have a callback or declare readOnly.
const unhandledInput = { label: 'Name', value: 'Ada' } satisfies InputFieldOptions
// @ts-expect-error A controlled value cannot have an autonomous initial value.
const mixedInput = { label: 'Name', value: 'Ada', onValueChange: change, defaultValue: 'Grace' } satisfies InputFieldOptions
// @ts-expect-error An unlabeled editor is inaccessible.
const unnamedInput = { defaultValue: 'Ada' } satisfies InputFieldOptions

const autonomousSelect = { label: 'Language', options: [], defaultValue: 'rust' } satisfies SelectOptions
const controlledSelect = { label: 'Language', options: [], value: 'rust', onValueChange: change,
  open: true, onOpenChange: toggle } satisfies SelectOptions

// @ts-expect-error A select cannot be controlled and autonomous at once.
const mixedSelect = { label: 'Language', options: [], value: 'rust', defaultValue: 'go' } satisfies SelectOptions
// @ts-expect-error Controlled open state requires a change callback.
const unhandledOpen = { label: 'Language', options: [], open: true } satisfies SelectOptions
// @ts-expect-error Controlled open state cannot have an autonomous initial state.
const mixedOpen = { label: 'Language', options: [], open: true, onOpenChange: toggle, defaultOpen: false } satisfies SelectOptions

void [autonomousInput, controlledInput, readOnlyInput, inputWithHelp, unhandledInput, mixedInput,
  unnamedInput, autonomousSelect, controlledSelect, mixedSelect, unhandledOpen, mixedOpen]
