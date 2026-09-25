/** @jsxImportSource @argui/react */
import type { ReactElement, ReactNode } from 'react'
import type { AspectRatioProps } from '../shared/foundation-b'
import { validAspectRatio } from '../shared/foundation-b'

/** Constrains its native child layout to a positive width-to-height ratio. */
export function AspectRatio(props: AspectRatioProps<ReactNode>): ReactElement {
  return <container width={props.width ?? 'fill'} aspect_ratio={validAspectRatio(props.ratio)}
    background={props.background ?? props.theme.surfaceRaised}
    radius={props.radius ?? props.theme.controlRadius} clip={props.clip ?? true}>
    {props.children}
  </container>
}
