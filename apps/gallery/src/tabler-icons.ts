import type { AssetRef } from '@argui/host'
import { mediaAssets } from './assets.generated'
import { tablerCatalogue } from './tabler-catalog.generated'

type IconBridge = { control(request: { kind: 'registerSvg'; id: number; svg: string }): void }
type IconGlobal = typeof globalThis & { __arguiBridge?: IconBridge }
const registered = new Set<number>()

/** Names available in the optional development Tabler catalogue. */
export const tablerIconNames = Object.keys(tablerCatalogue)

/** Returns a native SVG reference for a Tabler icon, registering it on first use.
 * The complete catalogue is included only when ARGUI_GALLERY_ALL_TABLER=1.
 * Bundled gallery SVGs remain available in ordinary release builds.
 */
export function tablerIcon(name: string): AssetRef {
  const bundled = mediaAssets[`tabler/${name}.svg` as keyof typeof mediaAssets]
  if (bundled) return bundled
  const icon = tablerCatalogue[name]
  if (!icon) throw new Error(`Tabler icon ${name} is unavailable; enable the dev catalogue or bundle this SVG`)
  if (!registered.has(icon.id)) {
    const bridge = (globalThis as IconGlobal).__arguiBridge
    if (!bridge) throw new Error('native Tabler icon bridge is unavailable')
    bridge.control({ kind: 'registerSvg', id: icon.id, svg: icon.svg })
    registered.add(icon.id)
  }
  return { kind: 'svg', id: icon.id }
}
