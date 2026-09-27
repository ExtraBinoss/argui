import { resolve } from 'node:path'
import { build, loadConfigFromFile, mergeConfig } from 'vite'

const [appArg, testArg, outArg] = process.argv.slice(2)
if (!appArg || !testArg || !outArg) throw new Error('usage: build-test APP TEST OUTPUT_DIR')
const app = resolve(appArg)
const test = resolve(testArg)
const outDir = resolve(outArg)
const sdkAliases = Object.fromEntries(
  ['host', 'solid', 'react', 'test'].map(name => [
    `@argui/${name}`, resolve(app, 'node_modules', '@argui', name),
  ]),
)
const loaded = await loadConfigFromFile(
  { command: 'build', mode: 'native' }, resolve(app, 'vite.config.ts'), app,
)
if (!loaded) throw new Error(`Cannot load ${resolve(app, 'vite.config.ts')}`)
const config = mergeConfig(loaded.config, {
  root: app,
  resolve: { alias: sdkAliases },
  build: {
    ssr: test,
    outDir,
    emptyOutDir: false,
    rollupOptions: { output: { entryFileNames: 'test.mjs' } },
  },
})
await build(config)
