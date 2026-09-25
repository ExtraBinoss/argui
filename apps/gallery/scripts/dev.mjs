import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const adapter = process.argv[2] ?? 'solid'
if (adapter !== 'solid' && adapter !== 'react') {
  console.error('usage: bun apps/gallery/scripts/dev.mjs [solid|react]')
  process.exit(2)
}

const root = resolve(fileURLToPath(new URL('../../../', import.meta.url)))
const child = spawn('cargo', ['run', '-p', 'argui-cli', '--', 'dev', 'apps/gallery'], {
  cwd: root,
  env: {
    ...process.env,
    CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR ?? resolve(root, 'target'),
    ARGUI_GALLERY_ENTRY: adapter,
  },
  stdio: 'inherit',
})
child.on('error', error => { console.error(error); process.exitCode = 1 })
child.on('exit', (code, signal) => { process.exitCode = code ?? (signal ? 130 : 1) })
