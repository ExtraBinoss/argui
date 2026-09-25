import type { JSX } from '@argui/solid/jsx-runtime'
import { createContext, useContext } from 'solid-js'
import type { Accessor, JSX as SolidJSX } from 'solid-js'

/** The writing direction applied to the provider's native subtree. */
export type WritingDirection = 'ltr' | 'rtl'

/** Props for a native direction scope; `direction` takes precedence over `dir`. */
export interface DirectionProviderProps {
  children: JSX.Element
  direction?: WritingDirection
  dir?: WritingDirection
  id?: string
}

const DirectionContext = createContext<Accessor<WritingDirection>>((): WritingDirection => 'ltr')

/** Applies native writing direction to descendants and makes it available through `useDirection`. */
export function DirectionProvider(props: DirectionProviderProps): JSX.Element {
  const direction = () => props.direction ?? props.dir ?? 'ltr'
  return <DirectionContext.Provider value={direction}>{(<container key={props.id ? `${props.id}-scope` : undefined}
    direction_scope={direction()}>{props.children}</container>) as unknown as SolidJSX.Element}</DirectionContext.Provider>
}

/** Returns the nearest writing direction accessor, defaulting to left-to-right. */
export function useDirection(): Accessor<WritingDirection> {
  return useContext(DirectionContext) ?? (() => 'ltr')
}
