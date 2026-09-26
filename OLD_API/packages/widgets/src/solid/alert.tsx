import type { AssetRef } from '@argui/host'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'

/** Visual and announcement priority for a themed native alert. */
export type AlertVariant = 'default' | 'destructive'

/** Props for a bordered alert with an optional icon and native text content. */
export interface AlertProps {
  /** Resolved application colors and sizing tokens. */
  theme: Palette
  /** Alert tone; destructive alerts use the destructive token. */
  variant?: AlertVariant
  /** Optional status icon shown before the alert copy. */
  icon?: AssetRef
  /** Primary line exposed as the alert's accessible name. */
  title?: string
  /** Supporting copy exposed as the alert's accessible description. */
  description?: string
  /** Custom native content, used instead of the built-in title and description. */
  children?: JSX.Element
}

/** Renders a themed, live native alert with optional icon and supporting copy. */
export function Alert(props: AlertProps): JSX.Element {
  const variant = () => props.variant ?? 'default'
  const tone = () => variant() === 'destructive' ? props.theme.destructive : props.theme.foreground
  return <rectangle width="fill" background={props.theme.surface}
    border_color={variant() === 'destructive' ? props.theme.destructive : props.theme.border}
    border_width={1} radius={props.theme.overlayRadius}>
    <row width="fill" padding={props.theme.overlayPadding} gap={12} align_items="start"
      role="alert" accessible_name={props.title} accessible_description={props.description}
      live="assertive">
      {props.icon ? <svg source={props.icon} color={tone()} width={18} height={18} /> : null}
      {props.children ?? <column width="fill" gap={4}>
        {props.title ? <AlertTitle theme={props.theme} text={props.title} variant={variant()} /> : null}
        {props.description ? <AlertDescription theme={props.theme} text={props.description} variant={variant()} /> : null}
      </column>}
    </row>
  </rectangle>
}

/** Props for a standalone alert heading. */
export interface AlertTitleProps {
  /** Resolved application colors. */
  theme: Palette
  /** Heading text shown in the alert. */
  text: string
  /** Heading color treatment. */
  variant?: AlertVariant
}

/** Renders the primary heading line for an alert. */
export function AlertTitle(props: AlertTitleProps): JSX.Element {
  const color = () => props.variant === 'destructive' ? props.theme.destructive : props.theme.foreground
  return <text width="fill" text={props.text} color={color()} font_size={props.theme.controlFontSize}
    weight={600} line_height={1.25} role="heading" level={3} />
}

/** Props for concise supporting alert copy. */
export interface AlertDescriptionProps {
  /** Resolved application colors. */
  theme: Palette
  /** Supporting text shown below the alert heading. */
  text: string
  /** Description color treatment. */
  variant?: AlertVariant
}

/** Renders supporting alert copy with the muted or destructive text token. */
export function AlertDescription(props: AlertDescriptionProps): JSX.Element {
  const color = () => props.variant === 'destructive' ? props.theme.destructive : props.theme.muted
  return <text width="fill" text={props.text} color={color()} font_size={13} line_height={1.4} />
}
