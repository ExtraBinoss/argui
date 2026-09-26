/** @jsxImportSource @argui/react */
import { createContext, createElement, useContext, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import type { ButtonGroupOptions } from '../shared/types'

const GroupContext = createContext<'horizontal' | 'vertical' | null>(null)

/** Returns the orientation of the nearest button group for joined child paint. */
export function useButtonGroup(): 'horizontal' | 'vertical' | null {
  return useContext(GroupContext)
}

/** Resets joined paint for controls rendered inside a floating portal. */
export function ButtonGroupBoundary(props: { children: ReactNode }): ReactElement {
  return createElement(GroupContext.Provider, { value: null, children: props.children })
}

/** Props for a labelled group of buttons, inputs, and separators. */
export type ButtonGroupProps = ButtonGroupOptions & { children: ReactNode }

/** Joins adjacent controls within one bordered, keyboard-traversable group. */
export function ButtonGroup(props: ButtonGroupProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const orientation = props.orientation ?? 'horizontal'
  const common = {
    id: props.id,
    role: 'group' as const,
    accessibleName: props.accessibleName,
    width: props.width,
    height: props.height ?? (orientation === 'horizontal' ? 40 : undefined),
    minWidth: props.minWidth,
    maxWidth: props.maxWidth,
    minHeight: props.minHeight,
    maxHeight: props.maxHeight,
    grow: props.grow,
    shrink: props.shrink,
    alignSelf: props.alignSelf ?? 'start',
    margin: props.margin,
    background: theme.surface,
    border: { width: 1, color: theme.border },
    focusBorderColor: theme.focusRing,
    radii: theme.radius,
    clip: true,
  }
  const fill = props.width !== undefined || props.alignSelf === 'stretch'
  const group = <rectangle {...common}>
    {orientation === 'horizontal'
      ? <row width={fill ? '100%' : undefined} height="100%" directionScope={props.directionScope}
          gap={0} alignItems="center">{props.children}</row>
      : <column width={fill ? '100%' : undefined} directionScope={props.directionScope}
          gap={0}>{props.children}</column>}
  </rectangle>
  return createElement(GroupContext.Provider, { value: orientation, children: group })
}

/** Props for the visible divider between joined controls. */
export type ButtonGroupSeparatorProps = { orientation?: 'horizontal' | 'vertical' }

/** Paints a one-pixel divider, perpendicular to the group by default. */
export function ButtonGroupSeparator(props: ButtonGroupSeparatorProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const group = useButtonGroup()
  const orientation = props.orientation ?? (group === 'vertical' ? 'horizontal' : 'vertical')
  return orientation === 'vertical'
    ? <rectangle width={1} height={24} shrink={0} background={theme.border} />
    : <rectangle width="100%" height={1} shrink={0} background={theme.border} />
}

/** Props for static text within a button group. */
export type ButtonGroupTextProps = { children?: ReactNode; render?: ReactNode }

/** Keeps supplementary text aligned with the group's controls. */
export function ButtonGroupText(props: ButtonGroupTextProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const content = props.render ?? (typeof props.children === 'string'
    ? <text color={theme.textMuted}>{props.children}</text> : props.children)
  return <container height={38} padding={{ start: 12, end: 12 }}>
    <row height="100%" alignItems="center">{content}</row>
  </container>
}
