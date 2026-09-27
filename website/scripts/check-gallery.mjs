import { access, readFile, readdir } from 'node:fs/promises'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const website = fileURLToPath(new URL('../', import.meta.url))
for (const adapter of ['solid', 'react']) {
  const directory = resolve(website, 'public/gallery', adapter)
  await access(resolve(directory, 'index.html'))
  const manifest = JSON.parse(await readFile(resolve(directory, 'assets.generated.json'), 'utf8'))
  if (manifest.version !== 1 || !Array.isArray(manifest.assets)) throw new Error(`Invalid ${adapter} manifest`)
  for (const asset of manifest.assets) await access(resolve(directory, 'assets', asset.path))
  const html = await readFile(resolve(directory, 'index.html'), 'utf8')
  const files = await readdir(directory, { recursive: true })
  if (!html.includes('argui-root') || !files.some(file => file.endsWith('.wasm'))) {
    throw new Error(`${adapter} gallery is missing its renderer. Run bun run gallery:build.`)
  }
}
console.log('[gallery] Both WebAssembly adapters and release assets are present.')
