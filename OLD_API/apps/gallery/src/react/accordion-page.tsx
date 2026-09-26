/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '../../../../packages/widgets/src/react/accordion'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates controlled accordion selection, item expansion and arrow navigation. */
export function AccordionPage(props: { theme: Palette }): ReactElement {
  const [value, setValue] = useState('product')
  return <column width="fill" gap={24}>
    <text text={`Open section: ${value || 'none'}`} color={props.theme.muted} font_size={13} />
    <Accordion id="surface-a-accordion" theme={props.theme} type="single" collapsible
      value={value} onValueChange={(next) => setValue(typeof next === 'string' ? next : next[0] ?? '')}
      label="Product help">
      <AccordionItem value="product"><AccordionTrigger>Product information</AccordionTrigger>
        <AccordionContent><text width="fill" text="A lightweight workspace for organizing projects, tasks, and shared notes." color={props.theme.foreground} font_size={14} /></AccordionContent>
      </AccordionItem>
      <AccordionItem value="delivery"><AccordionTrigger>Delivery details</AccordionTrigger>
        <AccordionContent><text width="fill" text="Standard delivery takes three to five business days. Express delivery arrives sooner." color={props.theme.foreground} font_size={14} /></AccordionContent>
      </AccordionItem>
      <AccordionItem value="returns"><AccordionTrigger>Returns</AccordionTrigger>
        <AccordionContent><text width="fill" text="Items may be returned within 30 days in their original condition." color={props.theme.foreground} font_size={14} /></AccordionContent>
      </AccordionItem>
    </Accordion>
    <text text="Focus the accordion and use Up or Down, Home or End to choose a trigger. Enter or Space toggles it."
      width="fill" color={props.theme.muted} font_size={13} />
  </column>
}
