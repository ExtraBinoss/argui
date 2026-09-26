import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '@argui/widgets/solid'
import { AlertDialog } from '../../../../packages/widgets/src/solid/alert-dialog'

/** Demonstrates safe default focus, Escape cancellation, and destructive confirmation. */
export function AlertDialogPage(props: { theme: Palette }): JSX.Element {
  const [deleted, setDeleted] = createSignal(false)
  return <column width="fill" gap={14}>
    <text width="fill" text="The cancel action receives initial focus. Escape cancels; confirming permanently removes this sample project."
      color={props.theme.muted} font_size={14} />
    <AlertDialog id="surface-d-alert" title="Delete project?" theme={props.theme}
      description="This action cannot be undone. The project and its saved settings will be removed."
      triggerLabel={deleted() ? 'Delete another project' : 'Delete project'}
      confirmLabel="Delete project" destructive onConfirm={() => setDeleted(true)} />
    <text text={deleted() ? 'Sample project deleted.' : 'Sample project is active.'}
      color={props.theme.foreground} font_size={13} />
  </column>
}
