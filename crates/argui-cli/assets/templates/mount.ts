import init, { ArguiWebHost } from '@argui/web-host/argui_app_web.js'
import type { NativeBridge } from '@argui/host'
import { mountGallery } from './main'

/** Mounts the app's WASM renderer and shared TSX scene into an HTML element. */
export async function mountArgui(elementId: string): Promise<() => void> {
  const root = document.getElementById(elementId)
  if (!root) throw new Error(`Missing Argui mount element #${elementId}`)
  root.addEventListener('argui:error', event => {
    root.textContent = `Argui could not render: ${String((event as CustomEvent).detail)}`
  })
  await init()
  const bridge = new ArguiWebHost(elementId)
  const adapter: NativeBridge = {
    contract: () => bridge.contract(),
    commit: operations => bridge.commit(operations),
    subscribe: callback => {
      const unsubscribe = bridge.subscribe(callback)
      return () => unsubscribe()
    },
    theme: {
      create: definition => bridge.themeCreate(definition),
      update: (id, patch) => bridge.themeUpdate(id, patch),
      subscribe: (id, callback) => bridge.themeSubscribe(id, callback),
      dispose: id => bridge.themeDispose(id),
    },
  }
  return mountGallery(adapter, adapter.contract().abiHash)
}
