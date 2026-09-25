import type { JSX } from '@argui/solid/jsx-runtime'
import type {
  AttachmentActionProps, AttachmentActionsProps as SharedAttachmentActionsProps,
  AttachmentContentProps as SharedAttachmentContentProps, AttachmentDescriptionProps,
  AttachmentGroupProps as SharedAttachmentGroupProps, AttachmentMediaProps,
  AttachmentProps as SharedAttachmentProps, AttachmentState, AttachmentTitleProps,
} from '../shared/chat-components'
import { Button } from './button'

export type AttachmentProps = SharedAttachmentProps<JSX.Element>
export type AttachmentContentProps = SharedAttachmentContentProps<JSX.Element>
export type AttachmentActionsProps = SharedAttachmentActionsProps<JSX.Element>
export type AttachmentGroupProps = SharedAttachmentGroupProps<JSX.Element>
export type { AttachmentState }

/** Presents an attachment card whose composed actions stay caller-controlled. */
export function Attachment(props: AttachmentProps): JSX.Element {
  const state = () => props.state ?? 'done'
  const vertical = () => props.orientation === 'vertical'
  const size = () => props.size ?? 'default'
  const padding = () => size() === 'xs' ? 6 : size() === 'sm' ? 8 : 10
  const progress = () => Number.isFinite(props.progress)
    ? Math.max(0, Math.min(100, props.progress!)) : undefined
  const status = () => state() === 'error' ? 'Attachment failed'
    : state() === 'uploading' ? 'Uploading attachment'
      : state() === 'processing' ? 'Processing attachment' : undefined
  const accentState = () => state() === 'uploading' || state() === 'processing'
  const errorState = () => state() === 'error'
  return <rectangle key={props.id} width="fit"
    background={errorState() ? `${props.theme.destructive}12` : props.theme.surface}
    border_color={errorState() ? props.theme.destructive : accentState() ? props.theme.accent : props.theme.border}
    border_width={1} radius={props.theme.controlRadius + 2} role="list_item" accessible_name={props.label}>
    <column width="fill" gap={6} padding={padding()}>
      {vertical()
        ? <column width="fill" align_items="center" gap={8}>{props.children}</column>
        : <row width="fill" gap={10} align_items="center">{props.children}</row>}
      {progress() !== undefined ? <column width="fill" gap={3}>
        {status() ? <text width="fill" text={status()!} color={errorState() ? props.theme.destructive : props.theme.muted}
          font_size={11} role="status" live={errorState() ? 'assertive' : 'polite'} /> : null}
        <rectangle key={`${props.id}-progress`} width={220} height={4} background={props.theme.surfaceRaised}
          radius={2} role="progress" accessible_name={`${props.label} progress`}
          numeric_value={progress()} minimum_value={0} maximum_value={100} value_step={1}>
          <rectangle width={220 * progress()! / 100} height="fill" background={errorState() ? props.theme.destructive : props.theme.accent} radius={2} />
        </rectangle>
      </column> : status() ? <text width="fill" text={status()!}
        color={errorState() ? props.theme.destructive : props.theme.muted} font_size={11}
        role="status" live={errorState() ? 'assertive' : 'polite'} /> : null}
    </column>
  </rectangle>
}

/** Renders a preview, image, or compact file glyph for an attachment. */
export function AttachmentMedia(props: AttachmentMediaProps<JSX.Element>): JSX.Element {
  const size = Number.isFinite(props.size) && (props.size ?? 0) > 0 ? props.size! : 40
  const variant = props.variant ?? 'icon'
  const media = props.source
    ? props.source.kind === 'image'
      ? <image source={props.source} alt={props.alt ?? ''} width="fill" height="fill" fit="cover" accessible_hidden={!props.alt} />
      : <svg source={props.source} alt={props.alt ?? ''} width="fill" height="fill" accessible_hidden={!props.alt} />
    : props.children ?? <text text="FILE" color={props.theme.muted} font_size={10} weight={700} />
  return <rectangle width={size} height={size} clip={variant === 'image'}
    background={props.theme.surfaceRaised} radius={props.theme.controlRadius}
    role={props.alt && !props.source ? 'image' : undefined} accessible_name={props.alt}
    accessible_hidden={!props.alt}>{media}</rectangle>
}

/** Groups an attachment title, description, and optional status. */
export function AttachmentContent(props: AttachmentContentProps): JSX.Element {
  return <column min_width={0} gap={3}>{props.children}</column>
}

/** Draws a compact, emphasized attachment filename or title. */
export function AttachmentTitle(props: AttachmentTitleProps): JSX.Element {
  return <text width="fill" text={props.text} color={props.theme.foreground} font_size={13} weight={600} />
}

/** Draws muted attachment details, with destructive color for errors. */
export function AttachmentDescription(props: AttachmentDescriptionProps): JSX.Element {
  return <text width="fill" text={props.text}
    color={props.state === 'error' ? props.theme.destructive : props.theme.muted} font_size={11} />
}

/** Arranges caller-owned attachment action buttons. */
export function AttachmentActions(props: AttachmentActionsProps): JSX.Element {
  return <row width="fit" wrap={true} gap={4} align_items="center">{props.children}</row>
}

/** Provides a native, keyboard-operable attachment action. */
export function AttachmentAction(props: AttachmentActionProps): JSX.Element {
  return <Button id={props.id} label={props.label} theme={props.theme}
    kind={props.kind ?? 'ghost'} size="sm" disabled={props.disabled} onClick={props.onClick} />
}

/** Provides a native attachment-open or attachment-selection action. */
export function AttachmentTrigger(props: AttachmentActionProps): JSX.Element {
  return <Button id={props.id} label={props.label} theme={props.theme}
    kind={props.kind ?? 'quiet'} size="sm" disabled={props.disabled} onClick={props.onClick} />
}

/** Displays attachments in a vertical list or horizontal native scroll viewport. */
export function AttachmentGroup(props: AttachmentGroupProps): JSX.Element {
  const orientation = props.orientation ?? 'horizontal'
  if (orientation === 'vertical') {
    return <column key={`${props.id}-group`} width="fill" gap={10} role="list" accessible_name={props.label ?? 'Attachments'}>
      {props.children}
    </column>
  }
  const height = Number.isFinite(props.height) && (props.height ?? 0) > 0 ? props.height! : 76
  return <flickable key={`${props.id}-scroll`} width="fill" height={height} scroll_x={true}
    role="list" accessible_name={props.label ?? 'Attachments'} orientation="horizontal"
    focusable={true} focus_on_tab_navigation={true}>
    <row width="fit" height="fill" gap={10} align_items="center">{props.children}</row>
  </flickable>
}
