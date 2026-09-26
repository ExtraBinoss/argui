import { expect, test } from 'bun:test'
import { execFileSync } from 'node:child_process'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { resolve } from 'node:path'
import builtin from '../src/contract.generated.json' with { type: 'json' }

const generator = resolve(import.meta.dirname, '../scripts/generate-custom-jsx.mjs')

test('a custom registry extends both JSX contracts with precise properties and events', () => {
  const directory = mkdtempSync(resolve(tmpdir(), 'argui-custom-jsx-'))
  try {
    const contract = { ...builtin, abiHash: 'custom-abi', natives: [...builtin.natives, {
      id: 900, name: 'Badge', properties: [
        { id: 1, name: 'id', valueType: 'String', readOnly: false },
        { id: 2, name: 'tone', valueType: 'String', readOnly: false, allowedValues: ['quiet', 'loud'] },
      ], events: [{ id: 1, name: 'click', eventType: 'click', payload: null }],
    }] }
    const input = resolve(directory, 'contract.json')
    writeFileSync(input, JSON.stringify(contract))
    execFileSync('bun', [generator, input, resolve(directory, 'generated')])
    for (const framework of ['react', 'solid']) {
      const declaration = readFileSync(resolve(directory, 'generated', `${framework}-custom.d.ts`), 'utf8')
      expect(declaration).toContain(`declare module '@argui/${framework}/jsx-runtime'`)
      expect(declaration).toContain('badge: {')
      expect(declaration).toContain('tone?: "quiet" | "loud"')
      expect(declaration).toContain("NativeEventPayload<'click'>")
    }
    expect(JSON.parse(readFileSync(resolve(directory, 'generated/contract.generated.json'), 'utf8')).abiHash).toBe('custom-abi')
  } finally { rmSync(directory, { recursive: true, force: true }) }
})
