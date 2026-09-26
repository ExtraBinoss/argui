import { mkdirSync, readFileSync, readdirSync, unlinkSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'

const root = resolve(import.meta.dirname, '..')
const check = process.argv.slice(2).includes('--check')
const adapters = ['host', 'react', 'solid']
let stale = false

/** Copies a flat SDK source directory and checks for stale or obsolete files. */
function syncDirectory(source, snapshot, pattern, preserved = new Set()) {
  if (!check) mkdirSync(snapshot, { recursive: true })
  const names = readdirSync(source).filter((name) => pattern.test(name))
  const expected = new Set(names)
  for (const name of names) {
    const current = readFileSync(join(source, name))
    const destination = join(snapshot, name)
    let embedded
    try { embedded = readFileSync(destination) } catch { embedded = null }
    if (embedded?.equals(current)) continue
    stale = true
    if (!check) writeFileSync(destination, current)
    console.error(`${check ? 'stale' : 'updated'} CLI SDK: ${source.slice(root.length + 1)}/${name}`)
  }
  for (const name of readdirSync(snapshot)) {
    if (preserved.has(name) || !pattern.test(name) || expected.has(name)) continue
    stale = true
    if (!check) unlinkSync(join(snapshot, name))
    console.error(`${check ? 'obsolete' : 'removed'} CLI SDK: ${source.slice(root.length + 1)}/${name}`)
  }
}

for (const adapter of adapters) {
  syncDirectory(
    join(root, 'packages', adapter, 'src'),
    join(root, 'crates/argui-cli/assets/sdk', adapter),
    /\.(?:ts|tsx|json)$/,
    new Set(['package.json']),
  )
}
syncDirectory(
  join(root, 'packages/host/scripts'),
  join(root, 'crates/argui-cli/assets/sdk/host/scripts'),
  /\.mjs$/,
)

if (check && stale) process.exitCode = 1
