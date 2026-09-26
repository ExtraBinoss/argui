# Migrating TSX applications to Argui v2

Argui v2 (workspace version 0.4.0) changes the public TSX contract and intentionally has no compatibility
aliases. The old gallery and component sources are preserved in `OLD_API/` for
reference; they are not part of the active build. Update an application by
moving its calls to the current generated Solid or React declarations and the
five maintained widgets: `Button`, `InputField`, `Select`, `Popover`, and
`VirtualList`.

## Public names and identity

All schema properties and events use `camelCase`, including properties declared
in Rust and serialized through the native wire. Rust implementation identifiers
remain idiomatic `snake_case`. The generated JSX declarations are the source of
truth for accepted names, values, and event payloads. Run `bun run generate:jsx`
after changing a native schema in this checkout.

Framework `key` is only for reconciliation. Set `id` when an element must be
addressed by an anchor, an accessibility relation, or a test. Widgets generate
stable internal IDs when `id` is omitted. Do not use visible text or a list
index as an ID.

## Text, values, and theme

Write ordinary text as `<text>Bonjour</text>` and use `text={value}` when an
explicit property is clearer. Do not provide both on one element. Text
primitives need a color appropriate to the current theme; the widget theme is
installed once at the application root and may receive partial local overrides.

`Button` displays ordinary text from `children` and uses `variant`. An icon-only
button requires `accessibleName`:

```tsx
<Button variant="default" onClick={save}>Save</Button>
<Button iconOnly accessibleName="Settings" onClick={openSettings}>{settingsIcon}</Button>
```

Value widgets use `value`, `defaultValue`, and `onValueChange`. Panels use
`open`, `defaultOpen`, and `onOpenChange`. Controlled props without the matching
callback are read-only. `Select` options have typed `{ value, label, disabled? }`
objects rather than a free-form string protocol.

## Layout and assets

`width` and `height` are preferred dimensions. Use `grow={1}` for remaining
flex space, `width="100%"` for a percentage of the parent, and `shrink={0}`
when an item should keep its preferred size. Use `minWidth={0}` when long flex
content must be allowed to shrink. `row`, `column`, `grid`, `container`, and
`scrollView` are the usual composition primitives; see the [layout
guide](ui/layout.md) for grid tracks, logical insets, and scroll bounds.

Icons belong to the application. Supply an icon pack or individual SVG files
to the application's asset configuration and reference them from TSX. The
development build can expose the configured pack; the release build discovers
statically referenced assets and includes only those used. An unresolved
dynamic asset lookup fails with a diagnostic so a release never silently
omits an icon.

## CLI

Create and run a new application with `argui init`, `argui check`, `argui dev`,
and `argui build release`. `init` includes Oxfmt in the generated application's
`package.json` and runs `bun install` when Bun is available. Use
`argui format [path]` to format its TSX or `argui format [path] --check` to
verify formatting without writes. Use `argui update --check` to inspect an available
CLI update, or `argui update` to install a SHA-256-verified release. Updating
the CLI does not rewrite an existing application's source.
