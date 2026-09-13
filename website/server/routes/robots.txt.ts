export default defineEventHandler((event) => {
  const origin = useRuntimeConfig(event).public.siteUrl.replace(/\/$/, '')
  const base = useRuntimeConfig(event).app.baseURL
  setHeader(event, 'content-type', 'text/plain; charset=utf-8')
  return `User-agent: *\nAllow: /\nDisallow: ${base}gallery/\n${origin ? `Sitemap: ${origin}/sitemap.xml\n` : ''}`
})
