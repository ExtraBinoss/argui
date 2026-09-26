import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { generateJSX } from '../../host/scripts/jsx-types.mjs'

const root = resolve(import.meta.dirname, '../../..')
const contract = JSON.parse(readFileSync(resolve(root, 'packages/host/src/contract.generated.json'), 'utf8'))
const destination = resolve(root, 'packages/react/src/jsx.generated.ts')
const content = generateJSX(contract, 'react')
if (process.argv.includes('--check')) {
  if (readFileSync(destination, 'utf8') !== content) throw new Error('Stale generated React JSX declarations')
} else writeFileSync(destination, content)
