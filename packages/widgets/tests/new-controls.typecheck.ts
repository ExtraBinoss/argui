import type { BooleanControlOptions, ProgressOptions, SliderOptions, TabsOptions } from '../src/shared/new-controls'

const onBoolean = (_value: boolean) => {}
const onNumber = (_value: number) => {}
const onTab = (_value: string) => {}

const autonomousCheckbox = { accessibleName: 'Save', defaultValue: true } satisfies BooleanControlOptions
const controlledSwitch = { accessibleName: 'Alerts', value: false, onValueChange: onBoolean } satisfies BooleanControlOptions
const readOnlyCheckbox = { accessibleName: 'Locked', value: true } satisfies BooleanControlOptions
const controlledTabs = { accessibleName: 'View', items: [], value: 'one', onValueChange: onTab } satisfies TabsOptions<string>
const localTabs = { accessibleName: 'View', items: [], defaultValue: 'one' } satisfies TabsOptions<string>
const slider = { accessibleName: 'Volume', min: 0, max: 100, step: 5, value: 25, onValueChange: onNumber } satisfies SliderOptions
const progress = { accessibleName: 'Upload', value: null } satisfies ProgressOptions

// @ts-expect-error Boolean controls require an accessible name.
const unnamedBoolean = { defaultValue: true } satisfies BooleanControlOptions
// @ts-expect-error A controlled value cannot also have a default value.
const mixedBoolean = { accessibleName: 'Save', value: true, defaultValue: false } satisfies BooleanControlOptions
// @ts-expect-error Tabs require an accessible name.
const unnamedTabs = { items: [] } satisfies TabsOptions<string>
// @ts-expect-error A controlled slider cannot also have a default value.
const mixedSlider = { accessibleName: 'Volume', value: 50, defaultValue: 40 } satisfies SliderOptions
// @ts-expect-error Progress requires a value, including null for indeterminate progress.
const missingProgress = { accessibleName: 'Upload' } satisfies ProgressOptions

void [autonomousCheckbox, controlledSwitch, readOnlyCheckbox, controlledTabs, localTabs,
  slider, progress, unnamedBoolean, mixedBoolean, unnamedTabs, mixedSlider, missingProgress]
