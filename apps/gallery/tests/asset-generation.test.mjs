import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'
import { generateAppAssets } from '../../../scripts/generate-app-assets.mjs'

function fixture(run) {
  const app = mkdtempSync(join(tmpdir(), 'argui-assets-'))
  try {
    mkdirSync(join(app, 'assets', 'icons'), { recursive: true })
    mkdirSync(join(app, 'assets', 'custom'), { recursive: true })
    mkdirSync(join(app, 'src'))
    for (const name of ['used', 'unused']) {
      writeFileSync(join(app, 'assets', 'icons', `${name}.svg`), `<svg><title>${name}</title></svg>`)
    }
    writeFileSync(join(app, 'assets', 'custom', 'brand.svg'), '<svg><title>brand</title></svg>')
    writeFileSync(join(app, 'src', 'main.tsx'), "import './page'\n")
    writeFileSync(join(app, 'src', 'page.tsx'),
      "import { mediaAssets as icons } from '../assets.generated'\nexport const icon = icons['chosen/used.svg']\n")
    writeFileSync(join(app, 'src', 'unused.tsx'),
      "import { mediaAssets } from '../assets.generated'\nexport const icon = mediaAssets['chosen/unused.svg']\n")
    const config = {
      packs: { chosen: 'icons' },
      files: { 'brand/mark.svg': 'custom/brand.svg' },
      entries: ['src/main.tsx'],
    }
    const path = join(app, 'assets.config.json')
    const generate = () => {
      writeFileSync(path, JSON.stringify(config))
      return generateAppAssets(path)
    }
    return run({ app, config, generate })
  } finally {
    rmSync(app, { recursive: true, force: true })
  }
}

test('release embeds only static keys reachable through imports; dev exposes the pack and own file', () => {
  fixture(({ app, generate }) => {
    const { release, development } = generate()
    assert.deepEqual(release.assets.map(asset => asset.key), ['chosen/used.svg'])
    assert.deepEqual(development.assets.map(asset => asset.key),
      ['brand/mark.svg', 'chosen/unused.svg', 'chosen/used.svg'])
    assert.ok(release.totalBytes < development.totalBytes)
    const rust = readFileSync(join(app, 'assets.generated.rs'), 'utf8')
    const releaseSection = rust.split('#[cfg(not(debug_assertions))]')[1]
    assert.match(releaseSection, /chosen\/used\.svg/)
    assert.doesNotMatch(releaseSection, /chosen\/unused\.svg|brand\/mark\.svg/)
    assert.deepEqual(JSON.parse(readFileSync(join(app, 'assets.generated.json'))), release)
    assert.deepEqual(JSON.parse(readFileSync(join(app, 'assets.dev.generated.json'))), development)
  })
})

test('dynamic asset access fails with a source location', () => {
  fixture(({ app, generate }) => {
    writeFileSync(join(app, 'src', 'page.tsx'),
      "import { mediaAssets } from '../assets.generated'\nexport const icon = mediaAssets[chosen]\n")
    assert.throws(generate, /page\.tsx:2: dynamic mediaAssets access cannot be packaged/)
  })
})

test('literal dynamic imports add reachable icons and variable module paths fail', () => {
  fixture(({ app, generate }) => {
    writeFileSync(join(app, 'src', 'main.tsx'), "export const page = import('./unused')\n")
    const { release } = generate()
    assert.deepEqual(release.assets.map(asset => asset.key), ['chosen/unused.svg'])
    writeFileSync(join(app, 'src', 'main.tsx'), "export const page = import(whichPage)\n")
    assert.throws(generate, /main\.tsx:1: dynamic module import cannot be scanned/)
  })
})

test('re-exported mediaAssets fail instead of silently dropping release icons', () => {
  fixture(({ app, generate }) => {
    writeFileSync(join(app, 'src', 'main.tsx'), "import { mediaAssets } from './icons'\nexport const icon = mediaAssets['chosen/used.svg']\n")
    writeFileSync(join(app, 'src', 'icons.ts'), "export { mediaAssets } from '../assets.generated'\n")
    assert.throws(generate, /icons\.ts:1: re-exporting mediaAssets hides release references/)
  })
})

test('asset paths cannot traverse or follow links outside their app', () => {
  fixture(({ app, config, generate }) => {
    config.files['escaped.svg'] = '../outside.svg'
    assert.throws(generate, /Unsafe source path/)
    delete config.files['escaped.svg']
    symlinkSync('/etc/hosts', join(app, 'assets', 'custom', 'outside.svg'))
    config.files['escaped.svg'] = 'custom/outside.svg'
    assert.throws(generate, /Asset source escapes root/)
  })
})
