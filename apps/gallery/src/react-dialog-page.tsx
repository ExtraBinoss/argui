/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { Button, Dialog, type Palette } from '@argui/widgets/react'

/** Shows the same centered native dialog through React. */
export function ReactDialogPage(props: { theme: Palette }): ReactElement {
  const [open, setOpen] = useState(false)
  return <column width="fill" gap={14}>
    <text width="fill" text="Open a modal dialog. The backdrop blurs the page, focus stays inside, and Escape closes it."
      color={props.theme.muted} font_size={14} />
    <rectangle width={440} height={230} radius={16} clip={true} background={props.theme.surfaceRaised}>
      <rectangle x={-24} y={37} width={152} height={152} radius={76} background="#f97316" />
      <rectangle x={100} y={-40} width={180} height={180} radius={90} background="#2563eb" />
      <rectangle x={273} y={89} width={142} height={142} radius={71} background="#e11d48" />
      <column x={18} y={18}>
        <Button id="dialog-open" label="Open dialog" theme={props.theme} kind="primary"
          onClick={() => setOpen(true)} />
      </column>
    </rectangle>
    <Dialog id="gallery-dialog" title="Welcome to Argui" theme={props.theme}
      open={open} onOpenChange={setOpen} blur={18}>
      <text width="fill" text="This modal stays centered. The colors behind it are blurred, and its button closes it."
        color={props.theme.muted} font_size={14} />
    </Dialog>
  </column>
}
