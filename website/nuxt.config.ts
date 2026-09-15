import { env } from 'node:process'
import catalogue from './app/data/catalogue.json'
import { docRoutes } from './app/data/doc-routes'
const baseURL = env.NUXT_APP_BASE_URL ?? '/'

export default defineNuxtConfig({
  compatibilityDate: '2026-09-13',
  devtools: { enabled: false },
  modules: ['@pinia/nuxt', '@nuxtjs/i18n'],
  css: ['~/assets/css/main.css'],
  runtimeConfig: { public: { siteUrl: '' } },
  app: {
    baseURL,
    head: {
      htmlAttrs: { lang: 'en' },
      link: [{ rel: 'icon', type: 'image/svg+xml', href: `${baseURL}favicon.svg` }],
      meta: [{ name: 'theme-color', content: '#f8fafc' }],
    },
  },
  i18n: {
    locales: [{ code: 'en', language: 'en', name: 'English', file: 'en.json' }],
    defaultLocale: 'en',
    strategy: 'prefix_except_default',
    detectBrowserLanguage: false,
  },
  nitro: {
    prerender: {
      concurrency: 2,
      routes: [
        '/',
        '/features',
        '/get-started',
        '/examples',
        '/docs',
        ...docRoutes.map((slug) => `/docs/${slug}`),
        '/components',
        '/sitemap.xml',
        '/robots.txt',
        ...catalogue.map((item) => `/components/${item.slug}`),
      ],
      ignore: ['/gallery', '/examples/ai-harness', '/examples/docs'],
    },
    compressPublicAssets: true,
  },
  routeRules: {
    '/get-started': { redirect: { to: '/docs/start/installation', statusCode: 301 } },
    '/docs/advanced/animation': {
      redirect: { to: '/docs/essentials/animation', statusCode: 301 },
    },
    '/gallery/**': {
      headers: { 'X-Robots-Tag': 'noindex', 'Cross-Origin-Resource-Policy': 'same-origin' },
    },
    '/examples/ai-harness/**': {
      headers: { 'X-Robots-Tag': 'noindex', 'Cross-Origin-Resource-Policy': 'same-origin' },
    },
    '/examples/docs/**': {
      headers: { 'X-Robots-Tag': 'noindex', 'Cross-Origin-Resource-Policy': 'same-origin' },
    },
  },
  typescript: { strict: true },
})
