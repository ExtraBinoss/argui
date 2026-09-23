import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

const entry = process.env.ARGUI_GALLERY_ENTRY
const react = entry === 'react'

export default defineConfig({
  root: import.meta.dirname,
  plugins: react ? [] : [solid({ solid: { moduleName: '@argui/solid', generate: 'universal' }, hot: false })],
  oxc: react ? { jsx: { runtime: 'automatic', importSource: '@argui/react' } } : undefined,
  define: react ? { 'process.env.NODE_ENV': JSON.stringify('production') } : undefined,
  ssr: { noExternal: react ? ['react', 'react-reconciler', 'scheduler', '@argui/react'] : ['solid-js'],
    resolve: { conditions: ['browser'] } },
  build: {
    ssr: react ? 'src/react-main.tsx' : 'src/main.tsx',
    outDir: 'dist',
    emptyOutDir: !react,
    target: 'es2022',
    rollupOptions: { output: { entryFileNames: react ? 'gallery-react-core.mjs' : 'gallery-core.mjs' } },
  },
})
