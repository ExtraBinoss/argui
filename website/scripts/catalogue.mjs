import { readFile, writeFile, mkdir, access } from 'node:fs/promises'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

export const root = fileURLToPath(new URL('../../', import.meta.url))
const read = (path) => readFile(resolve(root, path), 'utf8')
const sourceOverrides = {
  checkbox: 'selection',
  switch: 'selection',
  'radio-group': 'selection',
  textarea: 'input',
  layout: 'pages',
  motion: 'pages/motion',
  'liquid-glass': 'pages/liquid_glass',
  'scroll-shadow': 'pages/scroll_effects',
  typography: 'typography',
  'hot-reload': 'pages/hot_reload',
  i18n: 'pages/i18n',
  webview: 'pages/webview',
  'async-tasks': 'pages/async_tasks',
  editing: 'pages/editing',
  'custom-timeline': 'pages/timeline',
}
const extraWidgets = [
  [
    'command-palette',
    'Command palette',
    'Search and invoke actions from a keyboard-friendly palette.',
    'Actions',
  ],
  ['range', 'Range', 'Shared value, bounds, step and gesture behavior for range controls.', null],
  ['spinner', 'Spinner', 'A loading indicator that respects reduced motion.', 'Button'],
  [
    'split-pane',
    'Split pane',
    'Resizable panels with limits, pointer gestures and keyboard control.',
    null,
  ],
  [
    'tree-view',
    'Tree view',
    'Hierarchical navigation with virtualized rows and keyboard actions.',
    null,
  ],
  ['icons', 'Icons', 'Vector icons registered once and shared across widgets.', 'Button'],
]

export async function createCatalogue() {
  const navigation = await read('crates/argui-widget-gallery/src/navigation.rs')
  const table = await read('docs/widgets/shadcn.md')
  const manifest = await read('crates/argui-widgets/Cargo.toml')
  const features = new Set([...manifest.matchAll(/^([\w-]+) = \[/gm)].map((match) => match[1]))
  const values = (method) => {
    const block = navigation.split(`pub const fn ${method}(self)`)[1]?.split('\n    }')[0] ?? ''
    return Object.fromEntries(
      [...block.matchAll(/Self::(\w+)\s*=>\s*\{?\s*"([^"]*)"/g)].map((match) => [
        match[1],
        match[2],
      ]),
    )
  }
  const labels = values('label')
  const slugs = values('slug')
  const descriptions = values('description')
  const documented = Object.fromEntries(
    [...table.matchAll(/\|[^\n]+?\[([^\]]+)\]\(\.\.\/\.\.\/(crates\/[^)]+)\) \| `([^`]+)`/g)].map(
      (match) => [match[3], { api: match[1], source: match[2] }],
    ),
  )
  const items = Object.entries(slugs).map(([variant, slug]) => {
    const feature = features.has(slug) ? slug : null
    const file = sourceOverrides[slug] ?? slug.replaceAll('-', '_')
    const example = file.startsWith('pages')
    const source =
      documented[slug]?.source ??
      `crates/${example ? 'argui-widget-gallery' : 'argui-widgets'}/src/${file}.rs`
    return {
      slug,
      name: labels[variant],
      description: descriptions[variant],
      source,
      feature,
      category: feature ? 'Components' : 'Examples',
      gallery: labels[variant],
    }
  })
  for (const [slug, name, description, gallery] of extraWidgets) {
    items.push({
      slug,
      name,
      description,
      source:
        documented[slug]?.source ?? `crates/argui-widgets/src/${slug.replaceAll('-', '_')}.rs`,
      feature: slug,
      category: 'Components',
      gallery,
    })
  }
  for (const item of items) {
    if (!item.name || !item.description) throw new Error(`Missing gallery metadata: ${item.slug}`)
    await access(resolve(root, item.source))
  }
  return items.sort((a, b) => a.name.localeCompare(b.name, 'en'))
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const items = await createCatalogue()
  const destination = resolve(root, 'website/app/data/catalogue.json')
  await mkdir(dirname(destination), { recursive: true })
  await writeFile(destination, `${JSON.stringify(items, null, 2)}\n`)
  console.log(`Indexed ${items.length} components and examples with verified source paths.`)
}
