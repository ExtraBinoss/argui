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
const prefix = `[gallery:${adapter}]`

async function run(label, command, args, env = process.env) {
  const started = Date.now()
  console.log(`${prefix} ${label}…`)
  const heartbeat = setInterval(() => {
    console.log(`${prefix} ${label} still running (${Math.round((Date.now() - started) / 1000)}s).`)
  }, 15_000)
  try {
    const child = Bun.spawn([command, ...args], { cwd: root, env, stdout: 'inherit', stderr: 'inherit' })
    const code = await child.exited
    if (code !== 0) throw new Error(`${label} failed with exit code ${code}`)
    console.log(`${prefix} ${label} complete (${((Date.now() - started) / 1000).toFixed(1)}s).`)
  } finally {
    clearInterval(heartbeat)
  }
}

const manifest = await readFile(resolve(host, 'Cargo.toml'), 'utf8')
const bindgenVersion = manifest.match(/^wasm-bindgen = "=(\d+\.\d+\.\d+)"/m)?.[1]
const wasmPack = spawnSync('wasm-pack', ['--version'], { encoding: 'utf8' })
if (wasmPack.status !== 0) {
  console.error(`${prefix} wasm-pack is required to build the Web host. Install it with: cargo install wasm-pack --locked`)
  process.exit(1)
}
const installedBindgen = spawnSync('wasm-bindgen', ['--version'], { encoding: 'utf8' })
const useInstalledBindgen = bindgenVersion !== undefined
  && installedBindgen.status === 0
  && installedBindgen.stdout.trim() === `wasm-bindgen ${bindgenVersion}`

async function buildWebHost() {
  await run('Generating the JSX contract and CLI SDK', 'bun', ['run', 'generate:jsx'], {
    ...process.env,
    CARGO_TARGET_DIR: resolve(root, 'target/dev'),
  })
  console.log(`${prefix} ${useInstalledBindgen
    ? `Reusing installed wasm-bindgen ${bindgenVersion}.`
    : `wasm-pack will obtain wasm-bindgen ${bindgenVersion ?? 'required by the Web host'}.`}`)
  await run('Compiling the Rust WebAssembly host', 'wasm-pack', ['build', host, '--target', 'web', '--out-dir', 'pkg', '--dev',
    ...(useInstalledBindgen ? ['--mode', 'no-install'] : [])], {
    ...process.env,
    CARGO_TARGET_DIR: resolve(host, 'target/dev'),
  })
}

console.log(`${prefix} Starting the Web gallery.`)
await run('Generating gallery assets', 'bun', ['apps/gallery/scripts/generate-assets.mjs'])
await buildWebHost()

process.env.ARGUI_GALLERY_ENTRY = `web-${adapter}`
console.log(`${prefix} Starting the Vite development server…`)
const server = await createServer({
  configFile: resolve(root, 'apps/gallery/vite.config.ts'),
  server: { host: '127.0.0.1' },
})
await server.listen()
server.printUrls()
const localUrl = server.resolvedUrls?.local[0]
console.log(`${prefix} Ready${localUrl ? ` at ${new URL(page, localUrl)}` : `; open ${page}`}.
${prefix} Rust edits rebuild WebAssembly; TSX edits reload through Vite. Press Ctrl+C to stop.`)

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
      console.log(`${prefix} Rust source changed; rebuilding WebAssembly…`)
      try {
        await buildWebHost()
        server.moduleGraph.onFileChange(resolve(packageSource, 'argui_app_web.js'))
        server.moduleGraph.onFileChange(resolve(packageSource, 'argui_app_web_bg.wasm'))
        server.moduleGraph.invalidateAll()
        server.ws.send({ type: 'full-reload' })
        console.log(`${prefix} WebAssembly rebuilt; connected browsers reloaded.`)
      } catch (error) {
        console.error(`${prefix} WebAssembly rebuild failed; keeping the previous build.`, error)
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
