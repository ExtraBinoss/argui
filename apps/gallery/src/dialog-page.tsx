import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { Button, Dialog, type Palette } from '@argui/widgets/solid'

/** Shows a centered native dialog over a full-window blurred backdrop. */
export function DialogPage(props: { theme: Palette }): JSX.Element {
  const [open, setOpen] = createSignal(false)
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
      open={open()} onOpenChange={setOpen} blur={18}>
      <text width="fill" text="This modal stays centered. The colors behind it are blurred, and its button closes it."
        color={props.theme.muted} font_size={14} />
    </Dialog>
  </column>
}
