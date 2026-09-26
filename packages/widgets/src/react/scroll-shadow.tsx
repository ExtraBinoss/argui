/** @jsxImportSource @argui/react */
import type { ReactElement } from 'react'
import type { JSX } from '@argui/react/jsx-runtime'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'

/** Props for a native scroll viewport with overflow-aware edge fading. */
export type ScrollShadowProps = Omit<JSX.IntrinsicElements['scrollView'],
  'scrollX' | 'scrollY' | 'shadowWidth' | 'shadowIntensity' | 'orientation'> & {
  orientation?: 'vertical' | 'horizontal' | 'both'
  size?: number
}

/** Fades content at scrollable edges using native scroll metrics and theme size. */
export function ScrollShadow(props: ScrollShadowProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const { orientation = 'vertical', size, children, ...viewport } = props
  return <scrollView {...viewport}
    scrollX={orientation === 'horizontal' || orientation === 'both'}
    scrollY={orientation !== 'horizontal'}
    shadowWidth={size ?? theme.scrollShadowSize}>
    {children}
  </scrollView>
}
