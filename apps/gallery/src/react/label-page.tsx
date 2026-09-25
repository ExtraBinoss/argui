/** @jsxImportSource @argui/react */
import { type ReactElement } from 'react'
import { Label, Textarea, type Palette } from '@argui/widgets/react'

/** Shows a reusable label connected to a required multiline field. */
export function LabelPage(props: { theme: Palette }): ReactElement {
  return <column width="fill" gap={14}>
    <text width="fill" text="Labels expose a visible name and a semantic relation to the control with the matching key."
      color={props.theme.muted} font_size={14} />
    <Label htmlFor="label-example-input" text="Project name" theme={props.theme} required />
    <Textarea id="label-example-input" theme={props.theme} label="Project name"
      placeholder="Enter a project name" height={88} />
  </column>
}
