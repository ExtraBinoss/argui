import { splitProps } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { useTheme } from '@argui/solid'
import type { WidgetTheme } from '../shared/theme'

/** Props for a native scroll viewport with overflow-aware edge fading. */
export type ScrollShadowProps = Omit<JSX.IntrinsicElements['scrollView'],
  'scrollX' | 'scrollY' | 'shadowWidth' | 'shadowIntensity' | 'orientation'> & {
  orientation?: 'vertical' | 'horizontal' | 'both'
  size?: number
}

/** Fades content at scrollable edges using native scroll metrics and theme size. */
export function ScrollShadow(props: ScrollShadowProps): JSX.Element {
  const theme = useTheme<WidgetTheme>()
  const [local, viewport] = splitProps(props, ['orientation', 'size', 'children'])
  return <scrollView {...viewport}
    scrollX={local.orientation === 'horizontal' || local.orientation === 'both'}
    scrollY={local.orientation !== 'horizontal'}
    shadowWidth={local.size ?? theme().scrollShadowSize}>
    {local.children}
  </scrollView>
}
