import { createHash } from 'node:crypto'
import { readFileSync, readdirSync, realpathSync, statSync, writeFileSync } from 'node:fs'
import { dirname, extname, join, normalize, relative, resolve, sep } from 'node:path'

const root = process.cwd()
const sourceRoot = join(root, 'packages/widgets/src')
const version = readFileSync(join(root, 'Cargo.toml'), 'utf8')
  .match(/\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/)?.[1]
if (!version) throw new Error('Cannot read workspace package version')

const names = JSON.parse(readFileSync(join(root, 'components/catalog.json'), 'utf8')).sort()
if (new Set(names).size !== names.length || names.some((name) => !/^[a-z][a-z0-9-]*$/.test(name))) {
  throw new Error('Component catalog has duplicate or unsafe names')
}
for (const name of names) {
  for (const framework of ['solid', 'react']) {
    if (!readdirSync(join(sourceRoot, framework)).includes(`${name}.tsx`)) {
      throw new Error(`Missing ${framework}/${name}.tsx for catalog entry ${name}`)
    }
  }
}

const sourceFiles = new Map()

/** Resolves a widget source and all of its relative TS/TSX imports. */
function dependencies(entry, visited = new Set()) {
  const file = normalize(entry)
  if (visited.has(file)) return visited
  const full = resolve(sourceRoot, file)
  if (!full.startsWith(`${sourceRoot}${sep}`) || !['.ts', '.tsx'].includes(extname(full))) {
    throw new Error(`Unsafe widget dependency: ${file}`)
  }
  if (!realpathSync(full).startsWith(`${realpathSync(sourceRoot)}${sep}`)) {
    throw new Error(`Widget dependency escapes the source tree: ${file}`)
  }
  if (!statSync(full).isFile()) throw new Error(`Missing widget dependency: ${file}`)
  visited.add(file)
  const body = readFileSync(full)
  sourceFiles.set(file, createHash('sha256').update(body).digest('hex'))
  const text = body.toString('utf8')
  for (const [, imported] of text.matchAll(/\bfrom\s+['"](\.\.?\/[^'"]+)['"]/g)) {
    const base = resolve(dirname(full), imported)
    const resolved = ['.ts', '.tsx'].map((suffix) => `${base}${suffix}`)
      .find((candidate) => { try { return statSync(candidate).isFile() } catch { return false } })
    if (!resolved) throw new Error(`Cannot resolve ${imported} in ${file}`)
    dependencies(relative(sourceRoot, resolved).split(sep).join('/'), visited)
  }
  return visited
}

const components = Object.fromEntries(names.map((name) => [name, {
  version,
  source: Object.fromEntries(['solid', 'react'].map((framework) => [framework,
    `https://github.com/ExtraBinoss/argui/blob/v${version}/packages/widgets/src/${framework}/${name}.tsx`,
  ])),
  solid: [...dependencies(`solid/${name}.tsx`)].sort(),
  react: [...dependencies(`react/${name}.tsx`)].sort(),
}]))
const files = Object.fromEntries([...sourceFiles].sort(([a], [b]) => a.localeCompare(b)))
const registry = { version: 2, arguiVersion: version, files, components }
writeFileSync(join(root, 'components/registry.json'), `${JSON.stringify(registry, null, 2)}\n`)
console.log(`Registered ${names.length} paired components and ${sourceFiles.size} source files`)
