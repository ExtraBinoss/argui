/** @jsxImportSource @argui/react */
import { ApplicationServices, NativeHost, type NativeBridge } from '@argui/host'
import { createRoot } from '@argui/react'
import { WidgetAssetProvider } from '@argui/widgets/react'
import { isI18nBridgeAvailable, loadI18n, subscribeI18n, tr } from '@argui/i18n'
import enUS from './i18n/en-US.json'
import fr from './i18n/fr.json'
import { mediaAssets } from './assets.generated'
import { ReactGallery } from './react-gallery'

/** Mounts the React gallery through the same Argui native host and schema. */
export function mountReactGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  if (isI18nBridgeAvailable()) {
    loadI18n({ fallback: 'en-US', catalogs: { 'en-US': enUS, fr }, locale: 'en-US' })
  }
  const host = new NativeHost(bridge, expectedAbiHash)
  const services = new ApplicationServices(bridge)
  const updateTrayMenu = () => {
    void services.supports('menus', 'set').then((supported) => {
      if (supported) return services.setMenu([{ id: 'open-services', label: tr('trayOpenServices') }])
    }).catch((error: unknown) => { if (typeof console !== 'undefined') console.error('Tray menu:', error) })
  }
  if (isI18nBridgeAvailable()) updateTrayMenu()
  const unsubscribeLocale = isI18nBridgeAvailable() ? subscribeI18n(updateTrayMenu) : () => {}
  const root = createRoot(host, 'Column', { width: 'fill', height: 'fill' })
  root.render(
    <WidgetAssetProvider icons={{
      search: mediaAssets['tabler/search.svg'],
      chevronDown: mediaAssets['tabler/chevron-down.svg'],
      loader: mediaAssets['tabler/loader-2.svg'],
    }}>
      <ReactGallery services={services} />
    </WidgetAssetProvider>,
  )
  return () => { unsubscribeLocale(); services.dispose(); root.unmount() }
}

export { mountReactGallery as mountGallery }
