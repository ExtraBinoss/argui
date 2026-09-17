import { access, cp, mkdir, rm } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const root = fileURLToPath(new URL('../../', import.meta.url))
const destination = resolve(root, 'website/public/gallery')
const aiDestination = resolve(root, 'website/public/examples/ai-harness')
const gpuDestination = resolve(root, 'website/public/examples/gpu-canvas')
const docsDestination = resolve(root, 'website/public/examples/docs')
const astraDestination = resolve(root, 'website/public/examples/astra-editor')
const wasmEnvironment = { ...process.env }
const defaultBuildConcurrency = 2

const wasmBuilds = [
  {
    label: 'widget gallery',
    args: [
      'build',
      'crates/argui-widget-gallery',
      '--target',
      'web',
      '--release',
      '--out-dir',
      '../../web/widgets/pkg',
      '--all-features',
    ],
  },
  {
    label: 'AI harness example',
    args: [
      'build',
      'app_examples/fake-ai-harness',
      '--target',
      'web',
      '--release',
      '--out-dir',
      '../../web/examples/ai-harness/pkg',
    ],
  },
  {
    label: 'GPU canvas example',
    args: [
      'build',
      'app_examples/gpu-canvas',
      '--target',
      'web',
      '--release',
      '--out-dir',
      '../../web/examples/gpu-canvas/pkg',
    ],
  },
  {
    label: 'docs examples',
    args: [
      'build',
      'app_examples/docs-examples',
      '--target',
      'web',
      '--release',
      '--out-dir',
      '../../web/examples/docs/pkg',
    ],
  },
  {
    label: 'Astra Editor example',
    args: [
      'build',
      'app_examples/astra-editor',
      '--target',
      'web',
      '--release',
      '--out-dir',
      '../../web/examples/astra-editor/pkg',
    ],
  },
]

/**
 * Reads a positive integer setting without allowing malformed values to alter
 * the worker pool unexpectedly.
 *
 * @param {string | undefined} value The environment value to parse.
 * @param {number} fallback The value to use when the setting is invalid.
 * @returns {number} A positive integer suitable for a worker count.
 */
function positiveInteger(value, fallback) {
  if (!value || !/^\d+$/.test(value)) return fallback
  const parsed = Number(value)
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : fallback
}

/**
 * Runs one wasm-pack build while streaming its output directly to the caller.
 *
 * @param {{ label: string, args: string[] }} build The build command to run.
 * @returns {Promise<void>} Resolves when the build succeeds.
 * @throws {Error} If wasm-pack cannot start or exits unsuccessfully.
 */
function runWasmBuild(build) {
  return new Promise((resolveBuild, rejectBuild) => {
    const child = spawn('wasm-pack', build.args, {
      cwd: root,
      env: wasmEnvironment,
      stdio: 'inherit',
    })

    child.once('error', (error) => {
      rejectBuild(new Error(`${build.label} could not start: ${error.message}`, { cause: error }))
    })
    child.once('close', (code, signal) => {
      if (code === 0) {
        resolveBuild()
        return
      }
      const reason = signal ? `signal ${signal}` : `exit code ${code ?? 'unknown'}`
      const error = new Error(`${build.label} failed with ${reason}`)
      if (code !== null) error.exitCode = code
      rejectBuild(error)
    })
  })
}

/**
 * Runs wasm builds through a bounded worker pool so independent bundles can
 * overlap without spawning one process per bundle at once.
 *
 * @param {{ label: string, args: string[] }[]} builds The builds to execute.
 * @param {number} concurrency The maximum number of active wasm-pack processes.
 * @returns {Promise<void>} Resolves when every build succeeds.
 * @throws {Error} The first build error after active workers have drained.
 */
async function runWasmBuilds(builds, concurrency) {
  let nextBuild = 0
  let firstError

  async function worker() {
    while (firstError === undefined) {
      const build = builds[nextBuild]
      nextBuild += 1
      if (!build) return
      try {
        await runWasmBuild(build)
      } catch (error) {
        firstError = error
      }
    }
  }

  const workerCount = Math.min(concurrency, builds.length)
  await Promise.all(Array.from({ length: workerCount }, () => worker()))
  if (firstError) throw firstError
}

if (process.argv.includes('--check')) {
  try {
    await access(resolve(destination, 'pkg/argui_widget_gallery_bg.wasm'))
    await access(resolve(aiDestination, 'pkg/argui_example_ai_harness_bg.wasm'))
    await access(resolve(gpuDestination, 'pkg/argui_example_gpu_canvas_bg.wasm'))
    await access(resolve(docsDestination, 'pkg/argui_example_docs_bg.wasm'))
    await access(resolve(astraDestination, 'pkg/argui_example_astra_editor_bg.wasm'))
    await access(resolve(root, 'website/public/browser-shortcuts.js'))
  } catch {
    throw new Error(
      'Build the real gallery first: pnpm gallery:build (or pnpm gallery:copy for an existing web/widgets/pkg build).',
    )
  }
} else {
  if (!process.argv.includes('--copy')) {
    const concurrency = positiveInteger(
      process.env.ARGUI_WASM_BUILD_CONCURRENCY,
      defaultBuildConcurrency,
    )
    try {
      await runWasmBuilds(wasmBuilds, concurrency)
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      console.error(`WASM gallery build failed: ${message}`)
      process.exit(error?.exitCode ?? 1)
    }
  }
  await access(resolve(root, 'web/widgets/pkg/argui_widget_gallery_bg.wasm'))
  await access(resolve(root, 'web/examples/ai-harness/pkg/argui_example_ai_harness_bg.wasm'))
  await access(resolve(root, 'web/examples/gpu-canvas/pkg/argui_example_gpu_canvas_bg.wasm'))
  await access(resolve(root, 'web/examples/astra-editor/pkg/argui_example_astra_editor_bg.wasm'))
  await mkdir(destination, { recursive: true })
  await mkdir(aiDestination, { recursive: true })
  await mkdir(gpuDestination, { recursive: true })
  await mkdir(docsDestination, { recursive: true })
  await mkdir(astraDestination, { recursive: true })
  await rm(resolve(destination, 'pkg'), { recursive: true, force: true })
  await rm(resolve(aiDestination, 'pkg'), { recursive: true, force: true })
  await rm(resolve(gpuDestination, 'pkg'), { recursive: true, force: true })
  await rm(resolve(docsDestination, 'pkg'), { recursive: true, force: true })
  await rm(resolve(astraDestination, 'pkg'), { recursive: true, force: true })
  await cp(resolve(root, 'web/widgets/pkg'), resolve(destination, 'pkg'), { recursive: true })
  await cp(resolve(root, 'web/examples/ai-harness/pkg'), resolve(aiDestination, 'pkg'), {
    recursive: true,
  })
  await cp(resolve(root, 'web/examples/gpu-canvas/pkg'), resolve(gpuDestination, 'pkg'), {
    recursive: true,
  })
  await cp(resolve(root, 'web/examples/docs/pkg'), resolve(docsDestination, 'pkg'), {
    recursive: true,
  })
  await cp(resolve(root, 'web/examples/docs/index.html'), resolve(docsDestination, 'index.html'))
  await cp(resolve(root, 'web/examples/astra-editor/pkg'), resolve(astraDestination, 'pkg'), {
    recursive: true,
  })
  await cp(
    resolve(root, 'web/examples/astra-editor/index.html'),
    resolve(astraDestination, 'index.html'),
  )
  await cp(
    resolve(root, 'web/browser-shortcuts.js'),
    resolve(root, 'website/public/browser-shortcuts.js'),
  )
  console.log('The Argui WASM gallery and app examples are ready in public/.')
}
