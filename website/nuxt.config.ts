import docs from './app/data/docs-index.json'

const baseURL = process.env.NUXT_APP_BASE_URL ?? '/'

export default defineNuxtConfig({
  compatibilityDate: '2026-09-13',
  ssr: true,
  devtools: { enabled: false },
  modules: ['@pinia/nuxt'],
  css: ['~/assets/css/main.css'],
  app: {
    baseURL,
    head: {
      htmlAttrs: { lang: 'en' },
      link: [{ rel: 'icon', type: 'image/png', href: `${baseURL}argui-icon.png` }],
      meta: [{ name: 'theme-color', content: '#fafaf9' }],
    },
  },
  nitro: {
    prerender: {
      routes: ['/', '/components', '/docs', ...docs.map(page => `/docs/${page.slug}`)],
    },
  },
  typescript: { strict: true },
})
