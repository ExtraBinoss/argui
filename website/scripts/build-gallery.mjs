import { cp, mkdir, readFile, rename, rm, stat } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const website = fileURLToPath(new URL('../', import.meta.url))
const repository = resolve(website, '..')
const gallery = resolve(repository, 'apps/gallery')
const destination = resolve(website, 'public/gallery')
const buildEnvironment = {
  ...process.env,
  CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR ?? resolve(repository, 'target'),
  RUSTC_WRAPPER: process.env.RUSTC_WRAPPER ?? '',
}

function run(command, args, env = process.env) {
  return new Promise((done, fail) => {
    const child = spawn(command, args, { cwd: repository, env, stdio: 'inherit' })
    child.once('error', fail)
    child.once('close', code => code === 0 ? done() : fail(new Error(`${command} exited with ${code}`)))
  })
}

await run('bun', ['run', 'generate:assets'])
await run('wasm-pack', ['build', 'apps/gallery/web-host', '--target', 'web', '--out-dir', 'pkg', '--release'], buildEnvironment)
const manifest = JSON.parse(await readFile(resolve(gallery, 'assets.generated.json'), 'utf8'))
if (manifest.version !== 1 || !Array.isArray(manifest.assets)) throw new Error('Invalid gallery asset manifest')
await rm(destination, { recursive: true, force: true })
for (const adapter of ['solid', 'react']) {
  await run('bun', ['x', 'vite', 'build', '--config', 'apps/gallery/vite.config.ts', '--base', './'], {
    ...process.env,
    ARGUI_GALLERY_ENTRY: `web-${adapter}`,
  })
  const target = resolve(destination, adapter)
  await mkdir(target, { recursive: true })
  await cp(resolve(gallery, 'dist/web'), target, { recursive: true })
  if (adapter === 'react') await rename(resolve(target, 'react.html'), resolve(target, 'index.html'))
  await cp(resolve(gallery, 'assets.generated.json'), resolve(target, 'assets.generated.json'))
  for (const asset of manifest.assets) {
    const source = resolve(gallery, 'assets', asset.path)
    const output = resolve(target, 'assets', asset.path)
    if ((await stat(source)).size !== asset.bytes) throw new Error(`Asset changed after generation: ${asset.path}`)
    await mkdir(dirname(output), { recursive: true })
    await cp(source, output)
  }
}
console.log('[gallery] Built Solid and React WebAssembly galleries for the website.')
