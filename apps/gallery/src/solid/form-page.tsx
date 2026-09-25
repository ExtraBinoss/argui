import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { Form, FormControl, FormDescription, FormField, FormInput, FormItem, FormLabel, FormMessage, FormSubmit } from '../../../../packages/widgets/src/solid/form'

/** Demonstrates uncontrolled form state, required and custom validation, and valid submission feedback. */
export function FormPage(props: { theme: Palette }): JSX.Element {
  const [result, setResult] = createSignal('')
  return <column width="fill" gap={14}>
    <text width="fill" text="This native form owns string values and synchronous validation. Errors appear on submit, then update as fields change. No form-library or DOM submission service is required."
      color={props.theme.muted} font_size={13} />
    <Form id="foundation-i-bug-form" theme={props.theme} label="Bug report"
      defaultValues={{ title: '', description: '' }}
      onValuesChange={() => setResult('')}
      onSubmit={(values) => setResult(`Submitted “${values.title}” with ${values.description.length} description characters.`)}>
      <FormField name="title" label="Bug title" required
        description="Use at least five characters to identify the issue."
        validate={(value) => value.trim().length < 5 ? 'Bug title must be at least 5 characters.' : undefined}>
        <FormItem>
          <FormLabel />
          <FormControl><FormInput placeholder="Login button does not respond" /></FormControl>
          <FormDescription />
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField name="description" label="Description" required
        description="Include the steps, expected result, and what happened instead."
        validate={(value) => value.trim().length < 20 ? 'Description must be at least 20 characters.' : undefined}>
        <FormItem>
          <FormLabel />
          <FormControl><FormInput multiline height={104} placeholder="Describe how to reproduce the issue…" /></FormControl>
          <FormDescription />
          <FormMessage />
        </FormItem>
      </FormField>
      <FormSubmit label="Submit report" />
    </Form>
    {result() ? <text width="fill" text={result()} color={props.theme.accent} font_size={13} role="status" live="polite" /> : null}
  </column>
}
