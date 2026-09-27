export type SearchEntry = {
  title: string
  section: string
  kind: 'guide' | 'component' | 'page'
  href: string
  text: string
}

export type SearchHit = SearchEntry & { score: number; excerpt: string }

function normalize(value: string) {
  return value.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
}

function excerptFor(text: string, terms: string[]) {
  if (!text) return ''
  const lower = normalize(text)
  const positions = terms.map(term => lower.indexOf(term)).filter(position => position >= 0)
  const first = positions.length ? Math.min(...positions) : 0
  const initialStart = Math.max(0, first - 42)
  const start = initialStart ? Math.max(0, text.lastIndexOf(' ', initialStart) + 1) : 0
  const initialEnd = Math.min(text.length, start + 154)
  const wordEnd = text.lastIndexOf(' ', initialEnd)
  const end = initialEnd < text.length && wordEnd > start ? wordEnd : initialEnd
  return `${start ? '…' : ''}${text.slice(start, end).trim()}${end < text.length ? '…' : ''}`
}

export function searchEntries(entries: SearchEntry[], query: string, limit = 9): SearchHit[] {
  const phrase = normalize(query.trim())
  const terms = phrase.split(/\s+/).filter(Boolean)
  if (!terms.length) return []
  const matches: SearchHit[] = []
  for (const entry of entries) {
    const title = normalize(entry.title)
    const section = normalize(entry.section)
    const body = normalize(entry.text)
    if (!terms.every(term => title.includes(term) || section.includes(term) || body.includes(term))) continue
    let score = title === phrase ? 100 : title.startsWith(phrase) ? 50 : title.includes(phrase) ? 25 : 0
    if (section.includes(phrase)) score += 12
    for (const term of terms) {
      if (title.includes(term)) score += 15
      if (section.includes(term)) score += 5
      if (body.includes(term)) score += 1
    }
    matches.push({ ...entry, score, excerpt: excerptFor(entry.text, terms) })
  }
  return matches.sort((a, b) => b.score - a.score || a.title.localeCompare(b.title)).slice(0, limit)
}
