import { defineConfig } from 'vite'
import { resolve } from 'node:path'
import solid from 'vite-plugin-solid'

const entry = process.env.ARGUI_GALLERY_ENTRY
const react = entry === 'react'
const minimal = entry === 'minimal'
const allTabler = process.env.ARGUI_GALLERY_ALL_TABLER === '1'

export default defineConfig({
  root: import.meta.dirname,
  resolve: { alias: [{
    find: './tabler-catalog.generated',
    replacement: resolve(import.meta.dirname, 'src', allTabler
      ? 'tabler-catalog.generated.ts' : 'tabler-catalog.empty.ts'),
  }] },
  plugins: react ? [] : [solid({ solid: { moduleName: '@argui/solid', generate: 'universal' }, hot: false })],
  oxc: react ? { jsx: { runtime: 'automatic', importSource: '@argui/react' } } : undefined,
  define: react ? { 'process.env.NODE_ENV': JSON.stringify('production') } : undefined,
  ssr: { noExternal: react
    ? ['react', 'react-reconciler', 'scheduler', '@argui/react', '@argui/widgets']
    : ['solid-js', '@argui/widgets'],
    resolve: { conditions: ['browser'] } },
  build: {
    ssr: react ? 'src/react-main.tsx' : minimal ? 'src/minimal-main.tsx' : 'src/main.tsx',
    outDir: 'dist',
    emptyOutDir: !react,
    target: 'es2022',
    rollupOptions: { output: { entryFileNames: react ? 'gallery-react-core.mjs' : 'gallery-core.mjs' } },
  },
})
