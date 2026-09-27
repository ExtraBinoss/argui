export const components = [
  { slug: 'button', name: 'Button', description: 'Actions with variants, states, and accessible labels.' },
  { slug: 'button-group', name: 'Button group', description: 'Related actions with shared borders and selection.' },
  { slug: 'checkbox', name: 'Checkbox', description: 'A native checked state with keyboard and focus behavior.' },
  { slug: 'input-field', name: 'Input field', description: 'Text entry, labels, validation, and leading content.' },
  { slug: 'popover', name: 'Popover', description: 'Anchored content that dismisses cleanly.' },
  { slug: 'progress', name: 'Progress', description: 'Determinate and indeterminate progress feedback.' },
  { slug: 'select', name: 'Select', description: 'A choice menu backed by the Argui runtime.' },
  { slug: 'slider', name: 'Slider', description: 'Continuous values with pointer and keyboard control.' },
  { slug: 'switch', name: 'Switch', description: 'A compact on and off control.' },
  { slug: 'tabs', name: 'Tabs', description: 'Switch between related panels.' },
  { slug: 'tooltip', name: 'Tooltip', description: 'Brief contextual help on hover or focus.' },
  { slug: 'virtual-list', name: 'Virtual list', description: 'Large collections with bounded mounted content.' },
] as const

export type ComponentSlug = typeof components[number]['slug']
export type Adapter = 'solid' | 'react'

export function isComponentSlug(value: unknown): value is ComponentSlug {
  return typeof value === 'string' && components.some(component => component.slug === value)
}
