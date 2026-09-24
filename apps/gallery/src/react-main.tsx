/** @jsxImportSource @argui/react */
import { NativeHost, type NativeBridge } from '@argui/host'
import { createRoot } from '@argui/react'
import { WidgetAssetProvider } from '@argui/widgets/react'
import { isI18nBridgeAvailable, loadI18n } from '@argui/i18n'
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
  const root = createRoot(host, 'Column', { width: 'fill', height: 'fill' })
  root.render(
    <WidgetAssetProvider icons={{
      search: mediaAssets['tabler/search.svg'],
      chevronDown: mediaAssets['tabler/chevron-down.svg'],
      loader: mediaAssets['tabler/loader-2.svg'],
    }}>
      <ReactGallery />
    </WidgetAssetProvider>,
  )
  return () => root.unmount()
}

export { mountReactGallery as mountGallery }
