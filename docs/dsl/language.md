# Argui DSL language guide

The language is statically checked and deliberately excludes arbitrary Rust.
The semantic compiler resolves all runtime references to stable numeric IDs;
live and generated backends consume the same typed IR.

## Modules and declarations

Files are modules. Relative imports resolve lexically from the importing file:

```text
import { ProfileCard } from "./profile.argui"
import { Button, Column, Text } from "@argui/ui"
```

Only `export` declarations can be imported. Top-level declarations are
`struct`, `enum`, `component`, `theme`, `style`, and `effect`.

## Components

```text
export component ProfileCard {
    in property title: string
    in-out property selected: bool = false
    private property expanded: bool = false
    callback activate(title: string)
    slot content

    Column {
        Text { text: title }
        content
    }
}
```

Directions define the generated ABI:

- `in` accepts values from a parent or Rust.
- `out` exposes component-owned values.
- `in-out` supports explicit two-way binding.
- `private` survives compatible live reload but is absent from the public ABI.

Use `<=>` only with a writable property. Other assignments are reactive
one-way bindings.

## Expressions and handlers

Expressions include literals with units, property/local reads, struct fields,
arrays, unary/binary operations and conditionals. Event handlers are restricted
to callback calls, returns and assignments; they cannot embed Rust or perform
untracked side effects.

```text
Button {
    enabled: count < limit
    text: count == 0 ? "Start" : "Continue"
    on click { count += 1; changed(count) }
}
```

`asset("./logo.svg")`, `var(--accent)` and `tr("editor.save")` are typed
built-ins. `tr()` resolves through the application’s `argui-i18n` localizer and
falls back to its message ID when no translation is available.

## Conditional content and models

```text
if visible {
    Text { text: "Visible" }
} else {
    Text { text: "Hidden" }
}

for item in items key item.id {
    Text { text: item.label }
}
```

Stateful repeaters should provide a stable string or integer key. The compiler
warns when a key is absent. Source sites use structural identities rather than
line numbers, so unrelated edits do not reset focus, selection or child state.

## Themes and modes

```text
export theme AppTheme {
    --surface: color = #151821
    --accent: color = #735cff
    --space-md: length = 16px

    light { --surface: #ffffff }
    dark { --surface: #151821 }
}
```

Token references are resolved to typed IDs and derived-token cycles are
diagnosed. The live runtime switches a mode transactionally and invalidates
only expressions that depend on tokens.

Named styles declare typed property sets for a component/native target:

```text
export style PrimaryButton for Button {
    background: var(--accent)
    padding: var(--space-md)

    hovered { background: #856fff }
}
```

There is no CSS selector engine or specificity algorithm. Precedence is
deterministic: target defaults, theme defaults, component/named style, inline
properties, active state overrides, then animation presentation.

## States and animation

```text
states {
    disabled when !enabled { opacity: 0.5 }
    open when expanded { height: 300px }
}

animate height {
    spring { stiffness: 220.0 damping: 24.0 }
}
```

State conditions and animation parameters are ordinary typed expressions.
Animation slots are keyed by component instance, source site, property and
animation declaration, allowing compatible live edits to preserve presentation
state and velocity. The engine’s reduced-motion policy remains authoritative.

## External WGSL effects

```text
export effect Glow {
    shader: "../shaders/glow.wgsl"
    parameter intensity: float = 0.5
    parameter tint: color = #7c5cff
}
```

WGSL stays WGSL. Release builds and live packages validate Naga syntax, Argui’s
entry-point contract and the typed parameter ABI before exposing a generation.
An invalid replacement never destroys the previous renderer definition.

## Discover the component contract

Use `argui schema --json` to list every native primitive and official DSL
component. `argui schema Input --json` returns its properties, directions,
events, payloads, slots and documentation without requiring Rust source access.
