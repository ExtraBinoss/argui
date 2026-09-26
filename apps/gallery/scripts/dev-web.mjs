import { readFile, readdir } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { createServer } from 'vite'

const adapter = process.argv[2] ?? 'solid'
if (adapter !== 'solid' && adapter !== 'react') {
  console.error('usage: bun apps/gallery/scripts/dev-web.mjs [solid|react]')
  process.exit(2)
}

const root = resolve(fileURLToPath(new URL('../../../', import.meta.url)))
const host = resolve(root, 'apps/gallery/web-host')
const packageSource = resolve(host, 'pkg')
const page = adapter === 'react' ? '/react.html' : '/'

async function run(command, args, env = process.env) {
  const child = Bun.spawn([command, ...args], { cwd: root, env, stdout: 'inherit', stderr: 'inherit' })
  const code = await child.exited
  if (code !== 0) throw new Error(`${command} ${args[0]} failed with exit code ${code}`)
}

const manifest = await readFile(resolve(host, 'Cargo.toml'), 'utf8')
const bindgenVersion = manifest.match(/^wasm-bindgen = "=(\d+\.\d+\.\d+)"/m)?.[1]
const installedBindgen = spawnSync('wasm-bindgen', ['--version'], { encoding: 'utf8' })
const useInstalledBindgen = bindgenVersion !== undefined
  && installedBindgen.status === 0
  && installedBindgen.stdout.trim() === `wasm-bindgen ${bindgenVersion}`

async function buildWebHost() {
  await run('bun', ['run', 'generate:jsx'], {
    ...process.env,
    CARGO_TARGET_DIR: resolve(root, 'target/dev'),
  })
  await run('wasm-pack', ['build', host, '--target', 'web', '--out-dir', 'pkg', '--dev',
    ...(useInstalledBindgen ? ['--mode', 'no-install'] : [])], {
    ...process.env,
    CARGO_TARGET_DIR: resolve(host, 'target/dev'),
  })
}

await run('bun', ['apps/gallery/scripts/generate-assets.mjs'])
await buildWebHost()

process.env.ARGUI_GALLERY_ENTRY = `web-${adapter}`
const server = await createServer({
  configFile: resolve(root, 'apps/gallery/vite.config.ts'),
  server: { host: '127.0.0.1' },
})
await server.listen()
server.printUrls()
console.log(`Gallery Web (${adapter}): open ${page}; Rust edits rebuild WASM and reload the page.`)

const watchedRust = [resolve(root, 'Cargo.toml'), resolve(root, 'Cargo.lock'),
  resolve(host, 'Cargo.toml'), resolve(host, 'Cargo.lock'), resolve(host, 'src'),
  resolve(root, 'crates/argui-cli/assets/hosts/web/src')]
for (const crate of await readdir(resolve(root, 'crates'), { withFileTypes: true })) {
  if (!crate.isDirectory()) continue
  watchedRust.push(resolve(root, 'crates', crate.name, 'src'))
  watchedRust.push(resolve(root, 'crates', crate.name, 'Cargo.toml'))
}
server.watcher.add(watchedRust)

let timer
let rebuilding = false
let pending = false
async function rebuild() {
  if (rebuilding) { pending = true; return }
  rebuilding = true
  try {
    do {
      pending = false
      console.log('Argui: rebuilding WebAssembly…')
      try {
        await buildWebHost()
        server.moduleGraph.onFileChange(resolve(packageSource, 'argui_app_web.js'))
        server.moduleGraph.onFileChange(resolve(packageSource, 'argui_app_web_bg.wasm'))
        server.moduleGraph.invalidateAll()
        server.ws.send({ type: 'full-reload' })
        console.log('Argui: WebAssembly rebuilt; browser reloaded.')
      } catch (error) {
        console.error('Argui: WebAssembly rebuild failed; keeping the previous build.', error)
      }
    } while (pending)
  } finally {
    rebuilding = false
  }
}

server.watcher.on('all', (event, path) => {
  if (!['add', 'change', 'unlink'].includes(event)) return
  if (!path.endsWith('.rs') && !path.endsWith('Cargo.toml') && !path.endsWith('Cargo.lock')) return
  if (!watchedRust.some(watched => path === watched || path.startsWith(`${watched}/`))) return
  clearTimeout(timer)
  timer = setTimeout(() => void rebuild(), 200)
})
