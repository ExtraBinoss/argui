import { resolve } from 'node:path'
import { generateAppAssets } from '../../../scripts/generate-app-assets.mjs'

const { development, release } = generateAppAssets(resolve(import.meta.dirname, '..', 'assets.config.json'))
console.log(`[assets] Generated ${development.assets.length} development assets and ${release.assets.length} release assets.`)
