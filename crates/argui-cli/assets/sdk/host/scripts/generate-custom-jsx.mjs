#!/usr/bin/env bun
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { generateJSX } from './jsx-types.mjs'

const [contractPath, outputDirectory] = process.argv.slice(2)
if (!contractPath || !outputDirectory) {
  throw new Error('Usage: argui-generate-jsx CONTRACT.json OUTPUT_DIRECTORY')
}

const sourceContract = resolve(import.meta.dirname, '../src/contract.generated.json')
const snapshotContract = resolve(import.meta.dirname, '../contract.generated.json')
const builtins = JSON.parse(readFileSync(existsSync(sourceContract) ? sourceContract : snapshotContract, 'utf8'))
const contract = JSON.parse(readFileSync(resolve(contractPath), 'utf8'))
if (!contract.abiHash || !Array.isArray(contract.natives)) throw new Error('Invalid native schema contract')
const nativeNames = new Set()
const baseline = new Map(builtins.natives.map((native) => [native.name, native]))
const extra = []
for (const native of contract.natives) {
  if (nativeNames.has(native.name)) throw new Error(`Duplicate native primitive: ${native.name}`)
  nativeNames.add(native.name)
  const original = baseline.get(native.name)
  if (original) {
    if (JSON.stringify(original) !== JSON.stringify(native)) {
      throw new Error(`Custom contract changes built-in primitive ${native.name}`)
    }
  } else extra.push(native)
}
for (const name of baseline.keys()) if (!nativeNames.has(name)) throw new Error(`Custom contract omits built-in primitive ${name}`)
const output = resolve(outputDirectory)
mkdirSync(output, { recursive: true })
writeFileSync(resolve(output, 'contract.generated.json'), JSON.stringify(contract, null, 2) + '\n')
writeFileSync(resolve(output, 'react-custom.d.ts'), generateJSX({ natives: extra }, 'react', true))
writeFileSync(resolve(output, 'solid-custom.d.ts'), generateJSX({ natives: extra }, 'solid', true))
