# Native i18n in Rust, Solid and React

## Install the formatter before mounting a scene

`argui-i18n` owns locale negotiation and Fluent formatting in Rust.
`@argui/i18n` is its JavaScript bridge, with framework adapters at
`@argui/i18n/solid` and `@argui/i18n/react`. A custom QuickJS host must expose
`globalThis.__arguiBridge.i18n.load`, `.select` and `.tr` synchronously; merely
adding the npm package does not install the Rust localizer.

Call `loadI18n({ fallback, catalogs, locale })` before render/createRoot.
Each independent QuickJS session needs its own loaded localizer, including
settings, popups implemented as separate scenes, and retained hidden windows.
A scene using a different entry bundle still needs the same initialization.

```tsx
import { loadI18n } from '@argui/i18n'
import { useI18n } from '@argui/i18n/solid'

loadI18n({ fallback: 'en', locale: 'fr', catalogs: {
  en: { 'HUD-stop': 'Stop ({ $time })' },
  fr: { 'HUD-stop': 'Arrêter ({ $time })' },
} })

function RecordingAction() {
  const i18n = useI18n()
  const TR = (key: string, args?: Record<string, string | number | boolean>) =>
    i18n.tr(`HUD-${key}`, args)
  return <text>{TR('stop', { time: '00:42' })}</text>
}
```

Use the framework hook for reactive labels. Importing `tr` from the package
root does not subscribe Solid or React to locale changes. Create the hook
once in the component owner; do not call it from a reactive option mapper.
Translate option labels, placeholders, tooltips and accessible names too.
Keep list identities based on stable values, never translated labels.

## Import existing JSON catalogues at build time

`Catalog::from_json` accepts a flat `Record<string, string>` of Fluent patterns.
It does not accept nested Vue/i18next resources. Message IDs begin with an
ASCII letter, followed by letters, digits, underscores or hyphens; dots are
invalid. Convert `HUD.stop` to `HUD-stop`, detect flattening collisions, and
convert legacy `{time}` variables to Fluent `{ $time }` without editing the
source catalogues used by another renderer. Escape literal braces with Fluent
string expressions. Reject legacy plural syntax rather than displaying it.

Generate only the namespaces used by the native application. Feed the result
through the real Rust parser during a focused build test, for every locale.
A successful JSON parse or a fake JavaScript formatter cannot validate Fluent.

For counts, pass numbers, not preformatted strings, and use Fluent selects:

```text
items = { $count ->
    [one] One item
   *[other] { $count } items
}
```

Use the language's plural categories (including `few`/`many` where applicable).
A missing message uses the fallback locale; a present message with missing
variables or invalid formatting returns an error. Fallback is not a way to
hide malformed translations. Fluent may wrap interpolated arguments in bidi
isolators U+2068/U+2069; retain these in UI text and account for them in tests.

## Share preferences across native windows

When no valid explicit preference exists, negotiate the OS preferred language
list against the shipped catalogues, including regional and script variants.
Keep this automatic result out of storage so future launches can follow the OS.
A saved explicit selection takes precedence over system detection.

Persist one locale preference in the application's shared store and broadcast
the committed snapshot to every session. Hidden retained scenes must receive
updates and wake to process them. New windows read the current preference.
Call `selectLocale` in each session; updating only the settings component
leaves other windows in their old language.

Subscribe before issuing the initial asynchronous preference read. Track a
revision and a disposed flag so an old read cannot overwrite a newer event,
and unsubscribe when the scene is disposed. Avoid polling for language changes.
Use `useI18n().rtl` and `directionScope="rtl"` for right-to-left content; physical
popup cross offsets must follow that direction too.

## Check glyph coverage as well as translations

An embedded Latin font can omit Cyrillic, CJK and Devanagari. A text engine
constructed only with those embedded faces has no automatic OS font database.
Keep the application's font as the default family and load system or bundled
fallback fonts for the supported writing systems. Cache native system font
discovery across windows, while each window retains its own text engine.
Do not replace a missing glyph with an English label.

## Focused verification

Check actual catalogue parsing, interpolation and plural boundaries, invalid
catalogues and preserved fallback behavior. Mount the native adapter without a
window and change the locale: visible text, select options and accessible names
must update without losing node identities. Check a preference event arriving
before its initial read, disposal, a hidden scene and a newly created scene.
Desktop inspection is still needed for script-specific glyph rendering.
