import type { ButtonOptions } from '../src/shared/types'

const click = () => {}

const namedIcon = { size: 'icon', accessibleName: 'Favorite', onClick: click } satisfies ButtonOptions
const namedExplicitIcon = { iconOnly: true, accessibleName: 'Favorite', onClick: click } satisfies ButtonOptions
const textButton = { size: 'sm', onClick: click } satisfies ButtonOptions
const menuButton = { expanded: true, hasPopup: 'menu', onClick: click } satisfies ButtonOptions

// @ts-expect-error An icon-sized button must have an accessible name.
const unnamedIcon = { size: 'icon', onClick: click } satisfies ButtonOptions
// @ts-expect-error Explicit icon-only buttons must have an accessible name.
const unnamedExplicitIcon = { iconOnly: true, onClick: click } satisfies ButtonOptions
// @ts-expect-error Button height is selected by size, not an outer layout override.
const customHeight = { height: 52, onClick: click } satisfies ButtonOptions
// @ts-expect-error The native popup role uses listBox, not listbox.
const invalidPopup = { hasPopup: 'listbox', onClick: click } satisfies ButtonOptions

void namedIcon
void namedExplicitIcon
void textButton
void menuButton
void unnamedIcon
void unnamedExplicitIcon
void customHeight
void invalidPopup
