import { access, cp, mkdir, rm } from 'node:fs/promises'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const root = fileURLToPath(new URL('../../', import.meta.url))
const destination = resolve(root, 'website/public/gallery')
const exampleDestination = resolve(root, 'website/public/examples/ai-harness')
const wasmEnvironment = {
  ...process.env,
  CARGO_BUILD_JOBS: '6',
  BINARYEN_CORES: '6',
  CARGO_TARGET_DIR: resolve(root, 'target/website-wasm'),
}
if (process.argv.includes('--check')) {
  try {
    await access(resolve(destination, 'pkg/argui_widget_gallery_bg.wasm'))
    await access(resolve(exampleDestination, 'pkg/argui_example_ai_harness_bg.wasm'))
    await access(resolve(root, 'website/public/browser-shortcuts.js'))
  } catch {
    throw new Error(
      'Build the real gallery first: pnpm gallery:build (or pnpm gallery:copy for an existing web/widgets/pkg build).',
    )
  }
} else {
  if (!process.argv.includes('--copy')) {
    const build = spawnSync(
      'wasm-pack',
      [
        'build',
        'crates/argui-widget-gallery',
        '--target',
        'web',
        '--release',
        '--out-dir',
        '../../web/widgets/pkg',
        '--all-features',
      ],
      {
        cwd: root,
        stdio: 'inherit',
        env: wasmEnvironment,
      },
    )
    if (build.status !== 0) process.exit(build.status ?? 1)
    const exampleBuild = spawnSync(
      'wasm-pack',
      [
        'build',
        'app_examples/fake-ai-harness',
        '--target',
        'web',
        '--release',
        '--out-dir',
        '../../web/examples/ai-harness/pkg',
      ],
      { cwd: root, stdio: 'inherit', env: wasmEnvironment },
    )
    if (exampleBuild.status !== 0) process.exit(exampleBuild.status ?? 1)
  }
  await access(resolve(root, 'web/widgets/pkg/argui_widget_gallery_bg.wasm'))
  await access(resolve(root, 'web/examples/ai-harness/pkg/argui_example_ai_harness_bg.wasm'))
  await mkdir(destination, { recursive: true })
  await mkdir(exampleDestination, { recursive: true })
  await rm(resolve(destination, 'pkg'), { recursive: true, force: true })
  await rm(resolve(exampleDestination, 'pkg'), { recursive: true, force: true })
  await cp(resolve(root, 'web/widgets/pkg'), resolve(destination, 'pkg'), { recursive: true })
  await cp(resolve(root, 'web/examples/ai-harness/pkg'), resolve(exampleDestination, 'pkg'), {
    recursive: true,
  })
  await cp(
    resolve(root, 'web/browser-shortcuts.js'),
    resolve(root, 'website/public/browser-shortcuts.js'),
  )
  console.log('The Argui WASM gallery and app examples are ready in public/.')
}
