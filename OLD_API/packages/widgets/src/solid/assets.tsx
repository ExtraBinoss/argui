import { createContext, useContext } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { JSX as SolidJSX } from 'solid-js'
import type { WidgetIcons } from '../shared/types'

const WidgetIconsContext = createContext<WidgetIcons>({})

/** Supplies application-owned icons to widgets that have a built-in icon slot. */
export function WidgetAssetProvider(props: { icons: WidgetIcons; children: JSX.Element }): JSX.Element {
  return <WidgetIconsContext.Provider value={props.icons}>{props.children as SolidJSX.Element}</WidgetIconsContext.Provider>
}

/** Reads icons supplied by the nearest widget asset provider, if one exists. */
export function useWidgetIcons(): WidgetIcons {
  return useContext(WidgetIconsContext) ?? {}
}
