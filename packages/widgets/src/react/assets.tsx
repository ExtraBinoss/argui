/** @jsxImportSource @argui/react */
import { createContext, createElement, useContext, type ReactElement, type ReactNode } from 'react'
import type { WidgetIcons } from '../shared/types'

const WidgetIconsContext = createContext<WidgetIcons>({})

/** Supplies application-owned icons to widgets that have a built-in icon slot. */
export function WidgetAssetProvider(props: { icons: WidgetIcons; children: ReactNode }): ReactElement {
  return createElement(WidgetIconsContext.Provider, { value: props.icons }, props.children) as ReactElement
}

/** Reads icons supplied by the nearest widget asset provider. */
export function useWidgetIcons(): WidgetIcons {
  return useContext(WidgetIconsContext)
}
