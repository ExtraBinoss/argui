import { readdir, readFile, writeFile, mkdir, rm } from 'node:fs/promises'
import { dirname, join, relative, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Marked } from 'marked'
import { createHighlighter } from 'shiki'

const website = fileURLToPath(new URL('../', import.meta.url))
const repository = resolve(website, '..')
const docsRoot = resolve(repository, 'docs')
const indexOutput = resolve(website, 'app/data/docs-index.json')
const pagesOutput = resolve(website, 'app/data/doc-pages')
const searchOutput = resolve(website, 'public/search-index.json')
const sourceBase = 'https://github.com/ExtraBinoss/argui/blob/main/'
const sectionNames = {
  root: 'Start',
  ui: 'Interface',
  platform: 'Desktop & mobile',
  runtime: 'Runtime',
  rendering: 'Rendering',
  performance: 'Performance',
  contributing: 'Contributing',
  repo: 'Repository',
}
const highlighter = await createHighlighter({
  themes: ['github-light'],
  langs: ['tsx', 'typescript', 'rust', 'bash', 'powershell', 'toml', 'json', 'text'],
})
const codeLanguages = {
  tsx: ['tsx', 'TSX'],
  ts: ['typescript', 'TypeScript'],
  rust: ['rust', 'Rust'],
  sh: ['bash', 'Shell'],
  powershell: ['powershell', 'PowerShell'],
  toml: ['toml', 'TOML'],
  json: ['json', 'JSON'],
  text: ['text', 'Text'],
}

function escapeHtml(value) {
  return value.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')
}

function plainText(html) {
  return html.replace(/<[^>]*>/g, ' ').replace(/&(#x[\da-f]+|#\d+|amp|lt|gt|quot|apos|nbsp);/gi, (_, entity) => {
    const value = entity.toLowerCase()
    if (value.startsWith('#x')) return String.fromCodePoint(Number.parseInt(value.slice(2), 16))
    if (value.startsWith('#')) return String.fromCodePoint(Number.parseInt(value.slice(1), 10))
    return { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: ' ' }[value]
  }).replace(/\s+/g, ' ').trim()
}

function codeLabel(language, variant, defaultLabel) {
  if (language !== 'tsx' && language !== 'ts') return defaultLabel
  if (variant === 'solid') return `Solid ${defaultLabel}`
  if (variant === 'react') return `React ${defaultLabel}`
  if (language === 'tsx') return `Solid / React ${defaultLabel}`
  return defaultLabel
}

function slugFor(file) {
  const path = relative(docsRoot, file).split(sep).join('/')
  if (path === 'README.md') return ''
  if (path.endsWith('/README.md')) return path.slice(0, -10)
  return path.slice(0, -3)
}

function routeFor(file) {
  const slug = slugFor(file)
  return slug ? `/docs/${slug}` : '/docs'
}

function rewriteLink(href, source) {
  if (/^(https?:|mailto:|data:)/.test(href) || href.startsWith('#')) return href
  const [path, fragment = ''] = href.split('#', 2)
  const target = resolve(dirname(source), decodeURIComponent(path))
  const suffix = fragment ? `#${fragment}` : ''
  if (target.startsWith(`${docsRoot}${sep}`) || target === resolve(docsRoot, 'README.md')) {
    if (target.endsWith('.md') && !target.includes(`${sep}plans${sep}`) && !target.includes(`${sep}references${sep}`)) {
      return `${routeFor(target)}${suffix}`
    }
  }
  return `${sourceBase}${relative(repository, target).split(sep).join('/')}${suffix}`
}

async function markdownFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    if (entry.name === 'plans' || entry.name === 'references') continue
    const full = join(directory, entry.name)
    if (entry.isDirectory()) files.push(...await markdownFiles(full))
    else if (entry.name.endsWith('.md')) files.push(full)
  }
  return files
}

const pages = []
for (const file of await markdownFiles(docsRoot)) {
  const markdown = await readFile(file, 'utf8')
  const title = markdown.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? slugFor(file)
  const section = sectionNames[relative(docsRoot, file).split(sep)[0]] ?? sectionNames.root
  const headingCounts = new Map()
  const parser = new Marked({ gfm: true, breaks: false })
  parser.use({ renderer: {
    code({ text, lang }) {
      const [language = 'text', variant] = (lang ?? '').split(/[\s,]+/)
      const [grammar, defaultLabel] = codeLanguages[language] ?? ['text', 'Text']
      const label = codeLabel(language, variant, defaultLabel)
      const kind = variant === 'solid' || variant === 'react' ? variant : language === 'rust' ? 'rust' : language === 'tsx' ? 'both' : 'code'
      const html = highlighter.codeToHtml(text, { lang: grammar, theme: 'github-light' })
      return `<div class="code-block"><div class="code-block-label"><span class="code-block-badge code-block-badge-${kind}">${escapeHtml(label)}</span></div>${html}</div>`
    },
    heading({ tokens, depth }) {
      const text = this.parser.parseInline(tokens)
      const label = text.replace(/<[^>]+>/g, '')
      const base = label.toLowerCase().replace(/[^\p{L}\p{N}\s-]/gu, '').trim().replace(/\s+/g, '-')
      const count = headingCounts.get(base) ?? 0
      headingCounts.set(base, count + 1)
      const id = count ? `${base}-${count}` : base
      return `<h${depth} id="${escapeHtml(id)}">${text}</h${depth}>`
    },
    link({ href, title: linkTitle, tokens }) {
      const text = this.parser.parseInline(tokens)
      const destination = rewriteLink(href, file)
      const titleAttribute = linkTitle ? ` title="${escapeHtml(linkTitle)}"` : ''
      const external = /^https?:/.test(destination) ? ' target="_blank" rel="noopener noreferrer"' : ''
      return `<a href="${escapeHtml(destination)}"${titleAttribute}${external}>${text}</a>`
    },
  } })
  pages.push({
    slug: slugFor(file),
    title,
    section,
    source: relative(repository, file).split(sep).join('/'),
    html: await parser.parse(markdown),
  })
}
const startOrder = new Map(['', 'getting-started', 'cli', 'crates', 'architecture'].map((slug, index) => [slug, index]))
const sectionOrder = new Map(Object.values(sectionNames).map((section, index) => [section, index]))
pages.sort((a, b) => {
  const section = (sectionOrder.get(a.section) ?? 99) - (sectionOrder.get(b.section) ?? 99)
  if (section) return section
  if (a.section === 'Start' && b.section === 'Start') {
    const order = (startOrder.get(a.slug) ?? 99) - (startOrder.get(b.slug) ?? 99)
    if (order) return order
  }
  return a.slug.localeCompare(b.slug)
})
await rm(pagesOutput, { recursive: true, force: true })
const searchEntries = []
for (const page of pages) {
  const output = resolve(pagesOutput, `${page.slug || 'overview'}.json`)
  await mkdir(dirname(output), { recursive: true })
  await writeFile(output, JSON.stringify(page))
  const headings = [...page.html.matchAll(/<h([1-6]) id="([^"]+)">([\s\S]*?)<\/h\1>/g)]
  for (const [index, heading] of headings.entries()) {
    const body = page.html.slice(heading.index + heading[0].length, headings[index + 1]?.index ?? page.html.length)
    searchEntries.push({
      title: plainText(heading[3]),
      section: page.title,
      kind: 'guide',
      href: `${page.slug ? `/docs/${page.slug}` : '/docs'}#${heading[2]}`,
      text: plainText(body),
    })
  }
}
await mkdir(dirname(indexOutput), { recursive: true })
await writeFile(indexOutput, JSON.stringify(pages.map(({ slug, title, section }) => ({ slug, title, section }))))
await writeFile(searchOutput, JSON.stringify(searchEntries))
console.log(`[docs] Built ${pages.length} current guides and ${searchEntries.length} searchable sections.`)
