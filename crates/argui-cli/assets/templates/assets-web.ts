/** One app-owned encoded asset supplied to the WebAssembly renderer at startup. */
export interface WebAssetInput {
  key: string
  kind: 'svg' | 'image'
  id: number
  bytes: Uint8Array
}

interface ManifestEntry {
  key: string
  kind: 'svg' | 'image'
  id: number
  bytes: number
  path: string
}

function validEntry(value: unknown): value is ManifestEntry {
  if (!value || typeof value !== 'object') return false
  const item = value as Record<string, unknown>
  return typeof item.key === 'string'
    && (item.kind === 'svg' || item.kind === 'image')
    && Number.isSafeInteger(item.id) && Number(item.id) > 0
    && Number.isSafeInteger(item.bytes) && Number(item.bytes) >= 0
    && typeof item.path === 'string'
    && /^[A-Za-z0-9_./-]+$/.test(item.path)
    && item.path.split('/').every(part => part !== '' && part !== '.' && part !== '..')
}

/** Fetches the application's selected assets before mounting the WASM host.
 * `base` is the document URL, `development` chooses the complete pack manifest,
 * and `fetcher` is the browser fetch operation. Each returned byte buffer is
 * checked against the generated manifest. Throws on an invalid or missing asset.
 */
export async function loadWebAssets(
  base: string,
  development: boolean,
  fetcher: typeof fetch = fetch,
): Promise<WebAssetInput[]> {
  const name = development ? 'assets.dev.generated.json' : 'assets.generated.json'
  const response = await fetcher(new URL(name, base))
  if (!response.ok) throw new Error(`Could not load ${name}: HTTP ${response.status}`)
  const manifest: unknown = await response.json()
  if (!manifest || typeof manifest !== 'object' || (manifest as Record<string, unknown>).version !== 1
    || !Array.isArray((manifest as Record<string, unknown>).assets)) {
    throw new Error(`${name} has an invalid asset manifest`)
  }
  const entries = (manifest as { assets: unknown[] }).assets
  if (!entries.every(validEntry)) throw new Error(`${name} contains an invalid asset entry`)
  return Promise.all(entries.map(async entry => {
    const asset = await fetcher(new URL(`assets/${entry.path}`, base))
    if (!asset.ok) throw new Error(`Could not load ${entry.key}: HTTP ${asset.status}`)
    const bytes = new Uint8Array(await asset.arrayBuffer())
    if (bytes.byteLength !== entry.bytes) throw new Error(`${entry.key} changed after asset generation`)
    return { key: entry.key, kind: entry.kind, id: entry.id, bytes }
  }))
}
