import { createContext, useContext } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { useTheme } from '@argui/solid'
import type { WidgetTheme } from '../shared/theme'
import type { ButtonGroupOptions } from '../shared/types'

const GroupContext = createContext<'horizontal' | 'vertical' | null>(null)

/** Returns the orientation of the nearest button group for joined child paint. */
export function useButtonGroup(): 'horizontal' | 'vertical' | null {
  return useContext(GroupContext) ?? null
}

/** Resets joined paint for controls rendered inside a floating portal. */
export function ButtonGroupBoundary(props: { children: JSX.Element }): JSX.Element {
  return <GroupContext.Provider value={null}>{props.children}</GroupContext.Provider>
}

/** Props for a labelled group of buttons, inputs, and separators. */
export type ButtonGroupProps = ButtonGroupOptions & { children: JSX.Element }

/** Joins adjacent controls within one bordered, keyboard-traversable group. */
export function ButtonGroup(props: ButtonGroupProps): JSX.Element {
  const theme = useTheme<WidgetTheme>()
  const orientation = () => props.orientation ?? 'horizontal'
  return <GroupContext.Provider value={orientation()}>
    <rectangle id={props.id} role="group" accessibleName={props.accessibleName}
      width={props.width} height={props.height ?? (orientation() === 'horizontal' ? 40 : undefined)}
      minWidth={props.minWidth} maxWidth={props.maxWidth} minHeight={props.minHeight} maxHeight={props.maxHeight}
      grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf ?? 'start'} margin={props.margin}
      background={theme().surface} border={{ width: 1, color: theme().border }}
      focusBorderColor={theme().focusRing} radii={theme().radius} clip={true}>
      {orientation() === 'horizontal'
        ? <row width={props.width !== undefined || props.alignSelf === 'stretch' ? '100%' : undefined}
            height="100%" directionScope={props.directionScope} gap={0} alignItems="center">{props.children}</row>
        : <column width={props.width !== undefined || props.alignSelf === 'stretch' ? '100%' : undefined}
            directionScope={props.directionScope} gap={0}>{props.children}</column>}
    </rectangle>
  </GroupContext.Provider>
}

/** Props for the visible divider between joined controls. */
export type ButtonGroupSeparatorProps = { orientation?: 'horizontal' | 'vertical' }

/** Paints a one-pixel divider, perpendicular to the group by default. */
export function ButtonGroupSeparator(props: ButtonGroupSeparatorProps): JSX.Element {
  const theme = useTheme<WidgetTheme>()
  const group = useButtonGroup()
  const orientation = () => props.orientation ?? (group === 'vertical' ? 'horizontal' : 'vertical')
  return orientation() === 'vertical'
    ? <rectangle width={1} height={24} shrink={0} background={theme().border} />
    : <rectangle width="100%" height={1} shrink={0} background={theme().border} />
}

/** Props for static text within a button group. */
export type ButtonGroupTextProps = { children?: JSX.Element; render?: JSX.Element }

/** Keeps supplementary text aligned with the group's controls. */
export function ButtonGroupText(props: ButtonGroupTextProps): JSX.Element {
  const theme = useTheme<WidgetTheme>()
  const content = () => props.render ?? (typeof props.children === 'string'
    ? <text color={theme().textMuted}>{props.children}</text> : props.children)
  return <container height={38} padding={{ start: 12, end: 12 }}>
    <row height="100%" alignItems="center">{content()}</row>
  </container>
}
