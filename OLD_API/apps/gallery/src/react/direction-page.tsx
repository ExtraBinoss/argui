/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { ReactButton } from '../../../../packages/widgets/src/react/button'
import { DirectionProvider, useDirection, type WritingDirection } from '../../../../packages/widgets/src/react/direction'
import type { Palette } from '../../../../packages/widgets/src/shared/theme'

/** Demonstrates scoped native right-to-left direction and the direction context hook. */
export function DirectionPage(props: { theme: Palette }): ReactElement {
  const [direction, setDirection] = useState<WritingDirection>('ltr')
  return <column width="fill" gap={18}>
    <ReactButton id="direction-toggle" theme={props.theme} label={`Switch to ${direction === 'ltr' ? 'RTL' : 'LTR'}`}
      kind="secondary" onClick={() => setDirection(direction === 'ltr' ? 'rtl' : 'ltr')} />
    <DirectionProvider id="direction-provider-demo" direction={direction}>
      <rectangle width="fill" background={props.theme.surfaceRaised} border_color={props.theme.border}
        border_width={1} radius={props.theme.controlRadius}>
        <column width="fill" gap={10} padding={12}>
          <DirectionReadout theme={props.theme} />
          <row width="fill" gap={8}>
            <rectangle width={100} height={44} background={props.theme.accent} radius={props.theme.controlRadius}>
              <row width="fill" height="fill" align_items="center" justify_content="center">
                <text text="Source first" color={props.theme.accentText} font_size={12} />
              </row>
            </rectangle>
            <rectangle width={100} height={44} background={props.theme.surface} border_color={props.theme.border}
              border_width={1} radius={props.theme.controlRadius}>
              <row width="fill" height="fill" align_items="center" justify_content="center">
                <text text="Source second" color={props.theme.foreground} font_size={12} />
              </row>
            </rectangle>
          </row>
        </column>
      </rectangle>
    </DirectionProvider>
    <text width="fill" text="The provider sets the native direction scope for nested layout and portal content. Switch between LTR and RTL to see the source-order row follow writing direction."
      color={props.theme.muted} font_size={13} />
  </column>
}

function DirectionReadout(props: { theme: Palette }): ReactElement {
  const direction = useDirection()
  return <text text={`Direction from useDirection(): ${direction}`} color={props.theme.foreground} font_size={14} weight={600} />
}
