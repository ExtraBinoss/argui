/** @jsxImportSource @argui/react */
import { useId, useState, type ReactElement, type ReactNode } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import { buttonPaint, buttonRadius, buttonSize } from '../shared/button-paint'
import type { ButtonOptions } from '../shared/types'
import { useButtonGroup } from './button-group'

/** Props for the native React Button. */
export type ButtonProps = ButtonOptions & { children: ReactNode }

/** Renders a native button whose hover and press colors stay in the renderer. */
export function Button(props: ButtonProps): ReactElement {
  const generatedId = `argui-button-${useId().replaceAll(':', '')}`
  const [id] = useState(() => props.id ?? generatedId)
  const theme = useTheme<WidgetTheme>()
  const paint = buttonPaint(props.variant, theme, props.pressed)
  const sizing = buttonSize(props.size)
  const grouped = useButtonGroup() !== null
  const hasPreferredWidth = props.width !== undefined && props.width !== 'auto'
  const accessibleName = props.accessibleName ?? (typeof props.children === 'string' ? props.children : undefined)
  const content = typeof props.children === 'string'
    ? <text color={paint.foreground} fontSize={sizing.fontSize} weight={500}>{props.children}</text>
    : props.children

  return <focusScope
    id={id}
    role="button"
    pressedState={props.pressed}
    accessibleName={accessibleName}
    enabled={!props.disabled}
    mouseCursor={props.disabled ? 'notAllowed' : 'pointer'}
    expandable={props.expanded !== undefined}
    expanded={props.expanded}
    controls={props.controls}
    hasPopup={props.hasPopup}
    keyboardActivation="enterOrSpace"
    pressBounceScale={props.disabled || props.pressAnimation === false || props.expanded !== undefined || props.hasPopup !== undefined ? undefined : 0.97}
    onClick={() => { if (!props.disabled) props.onClick() }}
    width={props.width}
    minWidth={props.minWidth}
    maxWidth={props.maxWidth}
    grow={props.grow}
    shrink={props.shrink}
    alignSelf={props.alignSelf}
    margin={props.margin}
  >
    <rectangle
      width={hasPreferredWidth ? '100%' : props.iconOnly || sizing.icon ? sizing.height : undefined}
      height={sizing.height}
      padding={props.iconOnly || sizing.icon ? 0 : sizing.padding}
      background={paint.background}
      hoverBackground={paint.hover}
      border={{ width: 1, color: !grouped && props.variant === 'outline' ? theme.outlineBorder : 'transparent' }}
      radii={grouped ? 0 : props.rounded ? sizing.height / 2 : buttonRadius(props.size, theme.radius)}
      focusBorderColor={theme.focusRing}
      transitionMs={150}
      transitionTimingFunction="cubic-bezier(0.4, 0, 0.2, 1)"
      opacity={props.disabled ? 0.5 : 1}
    >
      <row width={hasPreferredWidth ? '100%' : undefined} height="100%" gap={sizing.gap} alignItems="center" justifyContent={props.contentAlign ?? 'center'}>
        {content}
      </row>
    </rectangle>
  </focusScope>
}
