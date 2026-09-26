import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
import { resolve } from 'node:path'

const entry = process.env.ARGUI_GALLERY_ENTRY
const react = entry === 'react' || entry === 'web-react'
const web = entry === 'web-solid' || entry === 'web-react'
const cliBuild = process.env.ARGUI_CLI_BUILD === '1'
const webPackage = resolve(import.meta.dirname, 'web-host/pkg')

export default defineConfig({
  root: import.meta.dirname,
  plugins: [
    ...(react ? [] : [solid({ solid: { moduleName: '@argui/solid', generate: 'universal' }, hot: false })]),
    ...(web ? [{
      name: 'argui-web-host-reload',
      hotUpdate({ file }: { file: string }) {
        if (file.startsWith(`${webPackage}/`)) return []
      },
    }] : []),
  ],
  oxc: react ? { jsx: { runtime: 'automatic', importSource: '@argui/react' } } : undefined,
  resolve: web ? { alias: { '@argui/web-host': webPackage } } : undefined,
  define: {
    __ARGUI_DEV_ASSETS__: JSON.stringify(process.env.ARGUI_GALLERY_DEV === '1'),
    ...(react ? { 'process.env.NODE_ENV': JSON.stringify('production') } : {}),
  },
  ssr: { noExternal: react
    ? ['react', 'react-reconciler', 'scheduler', '@argui/react', '@argui/widgets']
    : ['solid-js', '@argui/widgets'],
    resolve: { conditions: ['browser'] } },
  build: web ? { outDir: 'dist/web', target: 'es2022',
    rollupOptions: { input: resolve(import.meta.dirname, react ? 'react.html' : 'index.html') } } : {
    ssr: react ? 'src/react/main.tsx' : 'src/solid/main.tsx',
    outDir: 'dist',
    emptyOutDir: !react && !cliBuild,
    target: 'es2022',
    rollupOptions: { output: { entryFileNames: cliBuild ? 'app.mjs'
      : react ? 'gallery-react-core.mjs' : 'gallery-core.mjs' } },
  },
})
