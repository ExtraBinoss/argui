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

- `in` accepts values from a parent or Rust and is read-only inside a handler.
- `out` exposes component-owned values.
- `in-out` supports explicit two-way binding.
- `private` survives compatible live reload but is absent from the public ABI.

Callers can assign only `in` and `in-out` properties. Use `<=>` with a direct
writable property of the exact same type and a target that publishes changes.
Computed expressions, private/output child inputs and read-only destinations
are rejected. Other assignments are reactive one-way bindings.

Slots accept zero or more elements. Supply several slots by name:

```text
Dialog {
    open: true
    slot header { Text { content: "Delete item?" } }
    slot body { Text { content: "This action removes the selected item." } }
    slot actions { Button { text: "Cancel" } }
}
```

Declare a default with `slot header { Text { content: "Default" } }`. It is
rendered when the supplied slot has no visible elements. Bare slot references
project content and can forward it through another component's named slot.
Content retains the caller's properties and handlers. Implicit content goes
to the first declared slot; mixing implicit and named content is rejected.
Unknown and duplicate names, slot cycles, and content on a slotless component
produce diagnostics. Lazy list templates retain their existing keyed-repeater
contract; typed template parameters and required/single-child DSL slots are
not yet supported.

For `TextInput`, `value <=> draft` applies each accepted `on edit` replacement
to the controlled string. Edit ranges use UTF-8 byte offsets and always refer
to the value before that edit. `on edit` has no payload and runs after the
two-way value update; it is suitable for counters or other reactions that do
not need a copy of the full text. `Input` and `TextArea` in the standard library
use this path. `TextArea.changed()` fires after its value updates.

Use `on input(value)` on a native `TextInput` only when the handler needs the
complete edited string. That event supplies a copy of the whole value after
the edit. `on submit(value)` likewise supplies the submitted string.

## Expressions and handlers

Expressions include literals with units, property/local reads, struct fields,
arrays, unary/binary operations and conditionals. Event handlers are restricted
to callback calls, returns and assignments; they cannot embed Rust or perform
untracked side effects.

Callbacks may run only in handlers; bindings and animation expressions must be
pure. `&&`, `||`, and conditional expressions evaluate only the selected branch
in both backends. Assignments, compound operators, callback arguments and return
values are checked. A bare `return` exits a void handler; statements after a
return produce warnings in CLI and LSP.

Integers use signed 64-bit arithmetic and floats use 32-bit precision in both
backends. Compatible int-to-float conversions also apply inside optionals and
arrays. Integer overflow and division by zero are errors, rather than wrapping;
live reports the evaluation error, while generated AOT code panics with an
arithmetic diagnostic. This error-delivery difference remains part of the
current runtime boundary.

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
`str(value)` converts a scalar to text, `contains(text, fragment)` tests a
substring, and `solid(color)` creates a GPU brush.
`lower(text)` performs case-insensitive search preparation, and
`slice(text, start, length)` extracts characters for segmented inputs.
`range(count)` returns integers from zero up to (but excluding) `count`
(limited to 100,000 entries), which can feed a virtual repeater.
`hsv(hue, saturation, value, alpha)` creates a color with hue in degrees and
the other channels in the 0–1 range. `color_hex(color)` returns `#RRGGBBAA`;
`color_red/green/blue(color)` return 0–255 sRGB channel integers.

Gradient brushes use parallel color and normalized-offset arrays of any length
(at least two, with the same number of entries and ascending offsets). The
last argument selects `oklab`, `srgb`, or `linear-srgb` interpolation:

```text
linear_gradient([#ff0000, #00ff00, #0000ff], [0.0, 0.5, 1.0], angle, "oklab")
radial_gradient([#ff0000, #0000ff], [0.0, 1.0], 0.5, 0.5, 0.7, 0.7, "srgb")
conic_gradient([#ff0000, #0000ff], [0.0, 1.0], 0.5, 0.5, angle, "linear-srgb")
```

The numeric geometry values and colors can come from animated properties or
theme tokens. Text, TextEditor, and containers accept `selection_color` for a
solid highlight or `selection_fill` for a gradient, plus `selection_radius` for
the actual selected-text overlay. A container's selection style is inherited by
its text descendants; the default radius is 3 logical pixels. TextEditor additionally
accepts composable caret fill and primitive geometry; the gallery Input page
demonstrates bar, dot, and repeated-dot carets without a fixed style enum.
The `Text` primitive also accepts `line_height`, `font_style` (`normal`,
`italic`, `oblique`), `letter_spacing` in logical pixels, `underline` (`none`,
`single`, `double`), `strikethrough`, `text_align` (`start`, `end`, `left`,
`right`, `center`, `justify`), `line_clamp` (zero for no clamp), and
`text_overflow` (`clip` or `ellipsis_end`). These properties use the same text
shaper in live and generated builds.

## Drag handling

`TouchArea` exposes `drag_x(delta)` and `drag_y(delta)` for resize handles.
Each float is the total pointer displacement from the press in logical pixels.
Pan updates are coalesced once per frame, and release delivers the final
displacement. Record the starting size on `pointer_down`, then add the delta:

```text
TouchArea {
    on pointer_down { start_width = width }
    on drag_x(delta) { width = start_width + delta }
}
```

Use `moved` when every raw pointer movement needs an immediate callback.

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

Every repeater requires a stable string or integer key; omission is a compiler
error. Keys are scoped to the repeater and its enclosing keyed instance, so the
same key in two different outer rows denotes different state. Duplicate keys
are rejected at runtime (within the mounted window for virtual lists). Source
sites use structural identities rather than line numbers.

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
import { Rectangle } from "@argui/native"
export style AccentSurface for Rectangle {
    background: solid(var(--argui-primary))
    radius: 12.0
    hover { background: solid(var(--argui-primary-hover)) }
}

Rectangle { style: AccentSurface width: 120px height: 40px radius: 8.0 }
```

There is no CSS selector engine or specificity algorithm. Precedence is
deterministic: target defaults, theme defaults, component/named style, inline
properties, active state overrides, then animation presentation.
Apply exactly one matching style with `style: Name`. Styles can read constants,
pure built-ins and theme tokens. Native styles support `hover`, `pressed`,
`focus`, and `focus_visible`; state blocks on DSL-component styles are rejected.

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

An animation can bind `playing: running` to a boolean property. False freezes
its current presentation and stops requesting frames; true resumes the same
phase and excludes paused time. This works for numeric, dimension and color
timelines, keyframes, springs and state transitions. Omitting `playing` means
true. Unmounting the owner releases its retained animation slots.

## Backdrop filters

Every native visual surface, including `Rectangle`, `Text`, images, vector
paths, containers, and popup surfaces, accepts an ordered filter list:

```text
Rectangle {
    background: solid(#ffffff66)
    backdrop_filter: "blur(12px) saturate(150%) contrast(110%)"
}
```

The functions operate on pixels already painted behind the surface, in source
order. Supported CSS-style functions are `blur`, `brightness`, `contrast`,
`drop-shadow`, `grayscale`, `hue-rotate`, `invert`, `opacity`, `sepia`, and
`saturate`. Numbers, percentages, pixel lengths, and angle units follow their
CSS spellings. Argui also exposes its built-in `refraction` and `color-matrix`
filters. Use `none` to clear the list; `initial` and `unset` have the same
effect because DSL properties do not use the CSS cascade. Invalid syntax is an
adapter error.

The DSL cannot resolve a browser's external SVG `url()` filter or cascade
keywords such as `inherit` and `revert`, so these produce explicit errors.
For project WGSL effects, use an `effect` declaration with `scope: "backdrop"`.

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
