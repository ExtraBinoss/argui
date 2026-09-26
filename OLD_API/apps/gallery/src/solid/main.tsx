import { ApplicationServices, NativeHost, createThemeRuntime, type NativeBridge, type NativeNode } from '@argui/host'
import { render, useNativeHost } from '@argui/solid'
import { WidgetAssetProvider } from '@argui/widgets/solid'
import { isI18nBridgeAvailable, loadI18n, subscribeI18n, tr } from '@argui/i18n'
import enUS from '../i18n/en-US.json'
import fr from '../i18n/fr.json'
import { mediaAssets } from '../assets.generated'
import { Gallery } from './gallery'
import { galleryThemeDefinition, type GalleryTokens } from '../theme'

/** Mounts the Solid gallery into one canonical native Argui tree. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  if (isI18nBridgeAvailable()) {
    loadI18n({ fallback: 'en-US', catalogs: { 'en-US': enUS, fr }, locale: 'en-US' })
  }
  const host = new NativeHost(bridge, expectedAbiHash)
  const runtime = createThemeRuntime<GalleryTokens>(bridge, galleryThemeDefinition)
  const services = new ApplicationServices(bridge)
  const updateTrayMenu = () => {
    void services.supports('menus', 'set').then((supported) => {
      if (supported) return services.setMenu([{ id: 'open-services', label: tr('trayOpenServices') }])
    }).catch((error: unknown) => { if (typeof console !== 'undefined') console.error('Tray menu:', error) })
  }
  if (isI18nBridgeAvailable()) updateTrayMenu()
  const unsubscribeLocale = isI18nBridgeAvailable() ? subscribeI18n(updateTrayMenu) : () => {}
  useNativeHost(host)
  const root = host.createElement('Column')
  host.setProperty(root, 'width', 'fill')
  host.setProperty(root, 'height', 'fill')
  const dispose = render(() => (
    <WidgetAssetProvider icons={{
      search: mediaAssets['tabler/search.svg'],
      check: mediaAssets['tabler/check.svg'],
      x: mediaAssets['tabler/x.svg'],
      chevronDown: mediaAssets['tabler/chevron-down.svg'],
      chevronRight: mediaAssets['tabler/chevron-right.svg'],
      loader: mediaAssets['tabler/loader-2.svg'],
    }}>
      <Gallery services={services} runtime={runtime} />
    </WidgetAssetProvider>
  ) as NativeNode, root)
  host.setRoot(root)
  return () => { unsubscribeLocale(); services.dispose(); dispose(); runtime.dispose(); host.dispose() }
}
