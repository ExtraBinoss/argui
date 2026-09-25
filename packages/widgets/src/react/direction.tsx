/** @jsxImportSource @argui/react */
import { createContext, createElement, useContext, type ReactElement, type ReactNode } from 'react'

/** The writing direction applied to the provider's native subtree. */
export type WritingDirection = 'ltr' | 'rtl'

/** Props for a native direction scope; `direction` takes precedence over `dir`. */
export interface DirectionProviderProps {
  children: ReactNode
  direction?: WritingDirection
  dir?: WritingDirection
  id?: string
}

const DirectionContext = createContext<WritingDirection>('ltr')

/** Applies native writing direction to descendants and makes it available through `useDirection`. */
export function DirectionProvider(props: DirectionProviderProps): ReactElement {
  const direction = props.direction ?? props.dir ?? 'ltr'
  return createElement(DirectionContext.Provider, { value: direction },
    <container nativeKey={props.id ? `${props.id}-scope` : undefined} direction_scope={direction}>
      {props.children}
    </container>) as ReactElement
}

/** Returns the nearest writing direction, defaulting to left-to-right. */
export function useDirection(): WritingDirection {
  return useContext(DirectionContext)
}
