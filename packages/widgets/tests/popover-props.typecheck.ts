import type { PopoverOptions } from '../src/shared/types'

const uncontrolled: PopoverOptions = { trigger: 'Filters', contentWidth: 320 }
const controlled: PopoverOptions = { trigger: 'Filters', open: true, onOpenChange: () => {} }

// @ts-expect-error A controlled popover must provide its state-change callback.
const missingCallback: PopoverOptions = { trigger: 'Filters', open: true }

// @ts-expect-error A controlled popover cannot also specify uncontrolled initial state.
const mixedState: PopoverOptions = { trigger: 'Filters', open: true, onOpenChange: () => {}, defaultOpen: false }

void [uncontrolled, controlled, missingCallback, mixedState]
