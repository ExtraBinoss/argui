import { resolve } from 'node:path'
import { generateAppAssets } from '../../../scripts/generate-app-assets.mjs'

generateAppAssets(resolve(import.meta.dirname, '..', 'assets.config.json'))
