/** @jsxImportSource @argui/react */
import { useId, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import { buttonPaint, buttonSize } from '../shared/button-paint'
import type { ButtonOptions } from '../shared/types'
import { useButtonGroup } from './button-group'

/** Props for the native React Button. */
export type ButtonProps = ButtonOptions & { children: ReactNode }

/** Renders a native button whose hover and press colors stay in the renderer. */
export function Button(props: ButtonProps): ReactElement {
  const generatedId = `argui-button-${useId().replaceAll(':', '')}`
  const id = props.id ?? generatedId
  const theme = useTheme<WidgetTheme>()
  const paint = buttonPaint(props.variant, theme, props.pressed)
  const sizing = buttonSize(props.size)
  const grouped = useButtonGroup() !== null
  const hasPreferredWidth = props.width !== undefined && props.width !== 'auto'
  const content = typeof props.children === 'string'
    ? <text color={paint.foreground}>{props.children}</text>
    : props.children

  return <focusScope
    id={id}
    role="button"
    pressedState={props.pressed}
    accessibleName={props.accessibleName}
    enabled={!props.disabled}
    expandable={props.expanded !== undefined}
    expanded={props.expanded}
    controls={props.controls}
    hasPopup={props.expanded === undefined ? undefined : 'dialog'}
    keyboardActivation="enterOrSpace"
    onClick={() => { if (!props.disabled) props.onClick() }}
    width={props.width}
    height={props.height}
    minWidth={props.minWidth}
    maxWidth={props.maxWidth}
    minHeight={props.minHeight}
    maxHeight={props.maxHeight}
    grow={props.grow}
    shrink={props.shrink}
    alignSelf={props.alignSelf}
    margin={props.margin}
  >
    <rectangle
      width={hasPreferredWidth ? '100%' : props.iconOnly || sizing.icon ? sizing.height : undefined}
      height={grouped ? 38 : sizing.height}
      padding={props.iconOnly || sizing.icon ? 0 : sizing.padding}
      background={paint.background}
      hoverBackground={paint.hover}
      pressedBackground={paint.pressed}
      border={{ width: 1, color: !grouped && props.variant === 'outline' ? theme.outlineBorder : 'transparent' }}
      radii={grouped ? 0 : theme.radius}
      focusBorderColor={grouped ? undefined : theme.focusRing}
      pressedScale={props.pressAnimation === false ? undefined : 0.98}
      transitionSpring={props.pressAnimation !== false}
      opacity={props.disabled ? 0.5 : 1}
    >
      <row width={hasPreferredWidth ? '100%' : undefined} height="100%" gap={theme.spacing} alignItems="center" justifyContent={props.contentAlign ?? 'center'}>
        {content}
      </row>
    </rectangle>
  </focusScope>
}
