/** Navigation categories for individually implemented Solid and React component pages. */
export const widgetGroups = [
  {
    label: 'FOUNDATIONS',
    pages: ['Alert', 'Aspect Ratio', 'Avatar', 'Badge', 'Breadcrumb', 'Button Group', 'Card', 'Empty', 'Item', 'Kbd', 'Separator', 'Skeleton', 'Spinner'],
  },
  {
    label: 'FORM CONTROLS',
    pages: ['Checkbox', 'Field', 'Form', 'Input Group', 'Input OTP', 'Label', 'Native Select', 'Progress', 'Radio Group', 'Slider', 'Switch', 'Textarea', 'Toggle', 'Toggle Group'],
  },
  {
    label: 'SURFACES',
    pages: ['Accordion', 'Alert Dialog', 'Carousel', 'Collapsible', 'Combobox', 'Command', 'Context Menu', 'Direction', 'Drawer', 'Dropdown Menu', 'Hover Card', 'Menubar', 'Navigation Menu', 'Pagination', 'Resizable', 'Scroll Area', 'Sheet', 'Sidebar', 'Tabs', 'Tooltip'],
  },
  {
    label: 'CONTENT',
    pages: ['Attachment', 'Bubble', 'Calendar', 'Chart', 'Marker', 'Message', 'Message Scroller', 'Questionnaire', 'Sonner', 'Table', 'Toast'],
  },
] as const

export type WidgetPageName = typeof widgetGroups[number]['pages'][number]

/** The complete widget-page list, used for mounted pane visibility. */
export const widgetPages: readonly WidgetPageName[] = widgetGroups.flatMap((group) => group.pages)

/** Reports whether a gallery destination is backed by a widget demo. */
export function isWidgetPage(name: string): name is WidgetPageName {
  return (widgetPages as readonly string[]).includes(name)
}
