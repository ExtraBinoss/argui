# `@argui/i18n`

`@argui/i18n` loads flat JSON catalogs into Argui's Rust Fluent localizer. It
does not parse or format messages in JavaScript. The native QuickJS host must
provide `globalThis.__arguiBridge.i18n` before calling `loadI18n`.

```ts
import { loadI18n, selectLocale, tr } from '@argui/i18n'
import enUS from './locales/en-US.json'
import fr from './locales/fr.json'

loadI18n({ fallback: 'en-US', catalogs: { 'en-US': enUS, fr } })
const greeting = tr('greeting', { name: 'Ada' })
selectLocale('fr')
```

Catalog values are Fluent patterns, for example `"Hello, { $name }!"`.
Import JSON catalogs from the application bundle so development bundlers watch
them and native releases embed the same compiled catalog data.

## Framework adapters

`@argui/i18n/solid` exports `useI18n()`, returning `locale` and `rtl` accessors,
plus `tr` and `selectLocale`. `@argui/i18n/react` exports the same operations with
`locale` and `rtl` values from React's external-store subscription. Both adapters
refresh consumers after `selectLocale`; message formatting remains in Rust.
