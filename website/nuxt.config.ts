import { env } from 'node:process'
import catalogue from './app/data/catalogue.json'
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
        '/components',
        '/sitemap.xml',
        '/robots.txt',
        ...catalogue.map((item) => `/components/${item.slug}`),
      ],
      ignore: ['/gallery', '/examples/ai-harness'],
    },
    compressPublicAssets: true,
  },
  routeRules: {
    '/gallery/**': {
      headers: { 'X-Robots-Tag': 'noindex', 'Cross-Origin-Resource-Policy': 'same-origin' },
    },
    '/examples/ai-harness/**': {
      headers: { 'X-Robots-Tag': 'noindex', 'Cross-Origin-Resource-Policy': 'same-origin' },
    },
  },
  typescript: { strict: true },
})
