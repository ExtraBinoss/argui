import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { generateJSX } from '../../host/scripts/jsx-types.mjs'

const root = resolve(import.meta.dirname, '../../..')
console.log('[jsx] Exporting the native schema with Cargo (offline, all features); the first compile can take a while…')
const contract = JSON.parse(execFileSync('cargo', [
  'run', '--offline', '--all-features', '-p', 'argui-schema', '--example', 'export_contract',
], { cwd: root, encoding: 'utf8', stdio: ['inherit', 'pipe', 'inherit'] }))
const outputs = [
  ['packages/host/src/contract.generated.json', JSON.stringify(contract, null, 2) + '\n'],
  ['packages/solid/src/jsx.generated.ts', generateJSX(contract, 'solid')],
]
for (const [path, content] of outputs) {
  const destination = resolve(root, path)
  if (process.argv.includes('--check')) {
    if (readFileSync(destination, 'utf8') !== content) throw new Error(`Stale generated schema contract: ${path}`)
  } else writeFileSync(destination, content)
}
console.log(`[jsx] ${process.argv.includes('--check') ? 'Verified' : 'Generated'} Solid JSX declarations and the host contract.`)
