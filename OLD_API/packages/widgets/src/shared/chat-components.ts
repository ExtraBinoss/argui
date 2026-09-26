import type { AssetRef } from '@argui/host'
import type { Palette } from './theme'

/** Upload or processing state shown by an Attachment. */
export type AttachmentState = 'idle' | 'uploading' | 'processing' | 'error' | 'done'

/** Layout direction for one attachment card. */
export type AttachmentOrientation = 'horizontal' | 'vertical'

/** Size variant for an attachment card and its media area. */
export type AttachmentSize = 'default' | 'sm' | 'xs'

/** Props for a themed attachment card with optional transfer progress. */
export interface AttachmentProps<TChildren> {
  /** Stable key prefix for the attachment and progress indicator. */
  id: string
  /** Palette used for the card surface, status, and progress. */
  theme: Palette
  /** File metadata and any composed action controls. */
  children: TChildren
  /** Accessible card name, typically the attached filename. */
  label: string
  /** File transfer state. Defaults to `done`. */
  state?: AttachmentState
  /** Card layout direction. Defaults to horizontal. */
  orientation?: AttachmentOrientation
  /** Compactness of the media and card spacing. Defaults to `default`. */
  size?: AttachmentSize
  /** Optional transfer percentage, clamped to zero through one hundred. */
  progress?: number
}

/** Props for an attachment thumbnail or file icon. */
export interface AttachmentMediaProps<TChildren> {
  /** Palette used for the icon or preview background. */
  theme: Palette
  /** Vector or raster preview, when the caller has one. */
  source?: AssetRef
  /** Custom native visual shown instead of the source or default file glyph. */
  children?: TChildren
  /** Alternative name for a meaningful preview. */
  alt?: string
  /** Media box edge length in logical pixels. Defaults to 40. */
  size?: number
  /** Whether to render a square image preview rather than an icon tile. */
  variant?: 'icon' | 'image'
}

/** Props for the compact content column inside an Attachment. */
export interface AttachmentContentProps<TChildren> {
  /** Title, description, or status elements. */
  children: TChildren
}

/** Props for an attachment title line. */
export interface AttachmentTitleProps {
  /** Visible filename or attachment title. */
  text: string
  /** Palette used for the title. */
  theme: Palette
}

/** Props for muted or error attachment detail text. */
export interface AttachmentDescriptionProps {
  /** Visible file detail or current transfer status. */
  text: string
  /** Palette used for the description. */
  theme: Palette
  /** Error descriptions use the destructive color. */
  state?: AttachmentState
}

/** Props for a row of caller-owned attachment actions. */
export interface AttachmentActionsProps<TChildren> {
  /** One or more accessible native buttons. */
  children: TChildren
}

/** Props for one named attachment action button. */
export interface AttachmentActionProps {
  /** Stable native button key. */
  id: string
  /** Accessible and visible button label. */
  label: string
  /** Palette used by the native button. */
  theme: Palette
  /** Action invoked after pointer, keyboard, or accessibility activation. */
  onClick: () => void
  /** Whether the action is unavailable. */
  disabled?: boolean
  /** Button color treatment. Defaults to ghost. */
  kind?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive' | 'quiet' | 'link'
}

/** Props for a horizontally scrollable or vertical attachment list. */
export interface AttachmentGroupProps<TChildren> {
  /** Stable key for the attachment viewport. */
  id: string
  /** Attachment cards in display order. */
  children: TChildren
  /** Palette used for the native scroll indicator. */
  theme: Palette
  /** Accessible list name. Defaults to `Attachments`. */
  label?: string
  /** List flow direction. Defaults to horizontal. */
  orientation?: AttachmentOrientation
  /** Viewport height for a horizontal attachment list. Defaults to 76. */
  height?: number
}

/** Props for one themed message row. */
export interface MessageProps<TContent> {
  /** Stable key prefix for this message. */
  id: string
  /** Accessible name for the message list item. */
  label: string
  /** Whether this message is aligned as received or sent. */
  align?: 'start' | 'end'
  /** Optional caller-composed avatar, usually a MessageAvatar. */
  avatar?: TContent
  /** Header, body, and footer content for the message. */
  content: TContent
}

/** Props for an accessible vertical group of messages. */
export interface MessageGroupProps<TChildren> {
  /** Message rows in reading order. */
  children: TChildren
  /** Accessible conversation name. Defaults to `Messages`. */
  label?: string
  /** Space between message rows in logical pixels. Defaults to 14. */
  gap?: number
}

/** Props for a themed message content surface. */
export interface MessageContentProps<TChildren> {
  /** Rich text, attachments, or other native content. */
  children: TChildren
  /** Message direction controls its surface color. */
  variant?: 'incoming' | 'outgoing' | 'neutral'
  /** Palette used for the message surface and text. */
  theme: Palette
}

/** Props for a message sender and timestamp row. */
export interface MessageHeaderProps<TChildren> {
  /** Sender name, time, or other header content. */
  children: TChildren
}

/** Props for message status, reactions, or actions below the body. */
export interface MessageFooterProps<TChildren> {
  /** Timestamp, read state, reactions, or action controls. */
  children: TChildren
}

/** Accessibility or avatar props forwarded to the shared Avatar widget. */
export interface MessageAvatarProps {
  /** Palette used by the avatar and fallback surface. */
  theme: Palette
  /** Fallback initials shown without a source image. */
  fallback: string
  /** Alternative description for the sender portrait. */
  alt: string
  /** Optional image or SVG asset. */
  source?: AssetRef
  /** Avatar diameter. Defaults to 32. */
  size?: number
}

/** Normalized native scroll offsets from a flickable event. */
export interface MessageScrollOffset {
  /** Horizontal scroll position in logical pixels. */
  x: number
  /** Vertical scroll position in logical pixels. */
  y: number
}

/** Props for a fixed-height native message scrolling region. */
export interface MessageScrollerProps<TChildren> {
  /** Stable key prefix for the scroll viewport. */
  id: string
  /** Palette used for the viewport border and scrollbar. */
  theme: Palette
  /** Message content in reading order. */
  children: TChildren
  /** Accessible conversation name. Defaults to `Messages`. */
  label?: string
  /** Viewport width in pixels or fill; defaults to fill. */
  width?: number | 'fill'
  /** Fixed viewport height in logical pixels. Defaults to 320. */
  height?: number
  /** Receives the native logical-pixel scroll position when it changes. */
  onScroll?: (offset: MessageScrollOffset) => void
}

/** Reads the scroll position delivered by the host's flickable callback. */
export function messageScrollOffset(payload: unknown): MessageScrollOffset | undefined {
  if (typeof payload !== 'object' || payload === null) return undefined
  const value = payload as { offsetX?: unknown; offsetY?: unknown }
  if (typeof value.offsetX !== 'number' || !Number.isFinite(value.offsetX)
    || typeof value.offsetY !== 'number' || !Number.isFinite(value.offsetY)) return undefined
  return { x: value.offsetX, y: value.offsetY }
}
