/** @jsxImportSource @argui/react */
import { createContext, useContext, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'

const ButtonGroupContext = createContext(false)

/** Reports whether a button is inside a joined button group. */
export function useButtonGroupJoined(): boolean {
  return useContext(ButtonGroupContext)
}

/** Direction used to arrange controls within a button group. */
export type ButtonGroupOrientation = 'horizontal' | 'vertical'

/** Props for a semantic group of independently operable native controls. */
export interface ButtonGroupProps {
  /** Controls arranged as a toolbar-like group. */
  children: ReactNode
  /** Accessible name for the group of controls. */
  label?: string
  /** Stacking direction; horizontal is the default. */
  orientation?: ButtonGroupOrientation
  /** Space between controls in native layout units. */
  gap?: number
  /** Palette used for the joined group border and rounded corners. */
  theme?: Palette
}

/** Renders related controls in one labelled native group without merging their focus stops. */
export function ReactButtonGroup(props: ButtonGroupProps): ReactElement {
  const orientation = props.orientation ?? 'horizontal'
  const requestedGap = props.gap ?? 0
  const gap = Number.isFinite(requestedGap) ? Math.max(0, requestedGap) : 0
  const label = props.label
  const joined = !!props.theme && gap === 0
  const content = orientation === 'vertical'
    ? <column width="fit" gap={gap} align_items="stretch" role="group" accessible_name={label}>{props.children}</column>
    : <row width="fit" gap={gap} align_items="center" role="group" accessible_name={label}>{props.children}</row>
  return <ButtonGroupContext.Provider value={joined}>
    {joined ? <rectangle width="fit" background={props.theme!.surface}
      border_color={props.theme!.border} border_width={1}
      radius={props.theme!.controlRadius} clip={true}>{content}</rectangle> : content}
  </ButtonGroupContext.Provider>
}

/** Props for a themed text segment in a button group. */
export interface ButtonGroupTextProps {
  /** Resolved application colors. */
  theme: Palette
  /** Label shown in the group. */
  text: string
  /** Optional spoken label when the visual label is abbreviated. */
  label?: string
}

/** Renders a non-interactive, muted label alongside grouped controls. */
export function ReactButtonGroupText(props: ButtonGroupTextProps): ReactElement {
  return <rectangle height={36} background={props.theme.surfaceRaised} role="text"
    accessible_name={props.label ?? props.text}>
    <row height="fill" padding_left={props.theme.controlPadding} padding_right={props.theme.controlPadding}
      align_items="center">
      <text text={props.text} color={props.theme.foreground} font_size={props.theme.controlFontSize}
        weight={500} no_wrap={true} />
    </row>
  </rectangle>
}

/** Props for a decorative or semantic divider inside a button group. */
export interface ButtonGroupSeparatorProps {
  /** Resolved application colors. */
  theme: Palette
  /** Direction of the divider. Vertical is the default for a horizontal group. */
  orientation?: ButtonGroupOrientation
  /** Divider thickness in native layout units. */
  thickness?: number
  /** Whether assistive technology should skip the divider. */
  decorative?: boolean
}

/** Draws a theme-colored separator between button group segments. */
export function ReactButtonGroupSeparator(props: ButtonGroupSeparatorProps): ReactElement {
  const orientation = props.orientation ?? 'vertical'
  const requestedThickness = props.thickness ?? 1
  const thickness = Number.isFinite(requestedThickness) && requestedThickness > 0 ? requestedThickness : 1
  const decorative = props.decorative ?? true
  return <rectangle width={orientation === 'horizontal' ? 'fill' : thickness}
    height={orientation === 'horizontal' ? thickness : 'fill'} background={props.theme.border}
    role={decorative ? undefined : 'separator'} orientation={orientation}
    accessible_hidden={decorative} />
}
