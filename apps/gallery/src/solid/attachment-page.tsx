import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { AttachmentState } from '../../../../packages/widgets/src/shared/chat-components'
import type { Palette } from '@argui/widgets/solid'
import {
  Attachment, AttachmentAction, AttachmentActions, AttachmentContent,
  AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentTitle,
} from '../../../../packages/widgets/src/solid/attachment'

/** Demonstrates attachment status, progress, and caller-owned file actions. */
export function AttachmentPage(props: { theme: Palette }): JSX.Element {
  const [state, setState] = createSignal<AttachmentState>('uploading')
  const [progress, setProgress] = createSignal(68)
  const [status, setStatus] = createSignal('Upload is in progress.')
  const complete = () => { setState('done'); setProgress(100); setStatus('product-brief.pdf is ready.') }
  const retry = () => { setState('uploading'); setProgress(35); setStatus('Retrying product-brief.pdf.') }
  return <column width="fill" gap={14}>
    <text width="fill" text="Attachments render native metadata and progress. Open, retry, and remove actions are supplied by the host application; the widget does not open the system file picker itself."
      color={props.theme.muted} font_size={14} />
    <AttachmentGroup id="chat-attachments" theme={props.theme} label="Project attachments">
      <Attachment id="chat-attachment-upload" theme={props.theme} label="product-brief.pdf"
        state={state()} progress={progress()}>
        <AttachmentMedia theme={props.theme} alt="PDF document">
          <text text="PDF" color={props.theme.accent} font_size={11} weight={700} />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle text="product-brief.pdf" theme={props.theme} />
          <AttachmentDescription text={state() === 'done' ? '2.4 MB · Ready' : `${progress()}% · Uploading`}
            theme={props.theme} state={state()} />
          <AttachmentActions>
            {state() === 'error'
              ? <AttachmentAction id="chat-attachment-retry" label="Retry" theme={props.theme} kind="primary" onClick={retry} />
              : state() === 'done'
                ? <AttachmentAction id="chat-attachment-open" label="Open" theme={props.theme} onClick={() => setStatus('Open product-brief.pdf in the host application.')} />
                : <AttachmentAction id="chat-attachment-complete" label="Finish upload" theme={props.theme} onClick={complete} />}
          </AttachmentActions>
        </AttachmentContent>
      </Attachment>
      <Attachment id="chat-attachment-image" theme={props.theme} label="wireframe.png" state="done" size="sm">
        <AttachmentMedia theme={props.theme} alt="Wireframe image" variant="image" size={36}>
          <text text="IMG" color={props.theme.muted} font_size={10} weight={700} />
        </AttachmentMedia>
        <AttachmentContent>
          <AttachmentTitle text="wireframe.png" theme={props.theme} />
          <AttachmentDescription text="860 KB · Ready" theme={props.theme} />
        </AttachmentContent>
      </Attachment>
    </AttachmentGroup>
    <text text={status()} color={props.theme.foreground} font_size={13} role="status" live="polite" />
  </column>
}
