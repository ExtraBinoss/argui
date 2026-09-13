import catalogue from '../../app/data/catalogue.json'

export default defineEventHandler((event) => {
  const origin = useRuntimeConfig(event).public.siteUrl.replace(/\/$/, '')
  const paths = [
    '/',
    '/features',
    '/get-started',
    '/components',
    ...catalogue.map((item) => `/components/${item.slug}`),
  ]
  const escape = (value: string) =>
    value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('"', '&quot;')
  setHeader(event, 'content-type', 'application/xml; charset=utf-8')
  return `<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">${origin ? paths.map((path) => `<url><loc>${escape(origin + path)}</loc></url>`).join('') : ''}</urlset>`
})
