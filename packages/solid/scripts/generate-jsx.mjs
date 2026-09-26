import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { generateJSX } from '../../host/scripts/jsx-types.mjs'

const root = resolve(import.meta.dirname, '../../..')
const contract = JSON.parse(execFileSync('cargo', [
  'run', '--quiet', '--offline', '--all-features', '-p', 'argui-schema', '--example', 'export_contract',
], { cwd: root, encoding: 'utf8' }))
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
