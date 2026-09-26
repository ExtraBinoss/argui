import init, { ArguiWebHost } from '@argui/web-host/argui_app_web.js'
import type { NativeBridge } from '@argui/host'
import { loadWebAssets } from '../../../crates/argui-cli/assets/templates/assets-web'

/** Starts the shared TSX gallery with Argui's browser renderer. */
export async function mountWebGallery(
  elementId: string,
  mountGallery: (bridge: NativeBridge, expectedAbiHash: string) => () => void,
): Promise<() => void> {
  const root = document.getElementById(elementId)
  if (!root) throw new Error(`Missing Argui mount element #${elementId}`)
  root.addEventListener('argui:error', event => {
    root.textContent = `Argui could not render: ${String((event as CustomEvent).detail)}`
  })
  await init()
  const assets = await loadWebAssets(document.baseURI, true)
  const bridge = new ArguiWebHost(elementId, assets)
  const adapter: NativeBridge = {
    contract: () => bridge.contract(),
    commit: operations => bridge.commit(operations),
    subscribe: callback => bridge.subscribe(callback),
    theme: {
      create: definition => bridge.themeCreate(definition),
      update: (id, patch) => bridge.themeUpdate(id, patch),
      subscribe: (id, callback) => bridge.themeSubscribe(id, callback),
      dispose: id => bridge.themeDispose(id),
    },
  }
  return mountGallery(adapter, adapter.contract().abiHash)
}
