# Slint-style composition migration inventory

This inventory records the implementation boundary for
[the composition plan](SLINT_COMPOSITION_ARCHITECTURE_PLAN.md). The baseline is
the `codex/dsl-gallery-live` worktree on 2026-09-21. A listed native feature is
an engine mechanism, not proof that its DSL primitive is complete.

## Baseline boundary before migration

| Area | Current implementation | Composition gap |
| --- | --- | --- |
| Native schema | `argui-schema/src/builtin.rs` exposes `Container`, `Row`, `Column`, `Text`, `Pressable`, `TextEditor`, `PopoverPanel`, `Image`, `Svg`, `SwitchControl`, and `VList`. | The public primitive set needs `Rectangle`, `TouchArea`, `FocusScope`, `KeyBinding`, `PopupWindow`, `Flickable`, `TextInput`, and `Path`. `Pressable`, `PopoverPanel`, `SwitchControl`, and `VList` still combine engine behavior with control decisions. |
| Pointer and focus engine | `argui-ui` has typed pointer/key events, event propagation, pointer capture, `Interaction`, `FocusScope`, and retained visual states. | DSL element references cannot yet read hover, pressed, focus, or local pointer coordinates reactively. |
| Visual engine | `argui-paint`, `argui-render`, `argui-vector`, and `argui-ui::Element` already support brush fills, borders, radii, transforms, opacity, clipping, image/SVG assets, and generic effect layers. | The schema and DSL do not expose this complete surface uniformly; authored path geometry is missing. |
| Layout and scrolling | `Element` supports overflow/scroll configuration; the virtual-list mechanism computes a mounted row window. | `Container.scroll_y` chooses a scrollbar, while `VList` chooses scrollbar and edge-shadow visuals. `Flickable` and visual-free virtualization contracts are missing. |
| Overlays | `Element` can create portals and the UI engine has focus scopes and dismiss policies. | `PopoverPanel` fixes placement, dismissal, and paint in one native adapter; `PopupWindow` is not a composable DSL primitive. |
| Language | Parser, semantic model, typed IR, AOT codegen, live runtime, and hot-reload protocol exist in separate crates. | Native output properties, child references, typed pointer/key payloads, and their reactive AOT/live state reads need end-to-end contracts. |
| Standard library | Separate `.argui` modules exist for Button, Input, Switch, Card, Badge, Separator, and VirtualList. | Button styling is delegated to `Pressable`; Input uses `TextEditor` with control-level presentation; Switch uses `SwitchControl`; VirtualList uses `VList`. Slider and the remaining proof components have no `.argui` modules. |
| Gallery | `argui-widget-gallery` has many native Rust pages; the DSL fixture and examples exercise the current stdlib. | The six interactive proof areas in the composition plan need DSL-owned gallery pages and AOT/live parity checks. |

## Primitive contracts and ownership

1. `Rectangle` receives geometry, brush background, border width/color, radius,
   clip, opacity, and transforms. It never supplies a control role, click
   behavior, hover palette, or variant. The engine paints it; DSL bindings
   choose its current values.
2. `TouchArea` receives enablement, hit region, cursor, and event listeners.
   The engine supplies read-only hover/pressed and local pointer positions,
   capture, and event propagation. It does not paint or assign a Button role.
3. `FocusScope` and `KeyBinding` expose the existing focus engine and typed
   key events. They do not provide an Input, Menu, or Dialog appearance.
4. `PopupWindow` exposes portal anchoring, viewport placement, dismissal, and
   focus restoration. Its descendants provide the visible surface.
5. `Flickable` exposes viewport/content extent and scroll position. A separate
   virtual-window mechanism mounts visible model rows. DSL descendants draw
   scrollbars and edge shadows when desired.
6. `TextInput`, `Text`, `Image`, `Svg`, and `Path` expose low-level editing,
   text, asset, and geometry behavior. Frames, labels, validation, and other
   control presentation live in standard-library components.

## Migration order

1. Expose visual and interaction primitives through the canonical schema,
   including native output-property types and event payloads.
2. Carry native state reads and events through semantic checking, typed IR,
   generated Rust, live evaluation, and hot-reload invalidation.
3. Build and interactively verify Button and Slider from Rectangle, Text, and
   TouchArea. Keep the old paths until these are behaviorally complete.
4. Expose generic text input, portal, focus, scrolling, virtual-window, and
   path contracts. Compose the remaining proof controls in separate `.argui`
   files and gallery pages.
5. Migrate callers, remove old high-level schema adapters, then run parity,
   GUI, coverage, and the final quality gate.

The proof for each step is behavior in both AOT and live paths, not the mere
presence of a schema name or `.argui` wrapper.

## Implemented boundary

The public schema now exposes `Rectangle`, `TouchArea`, `FocusScope`,
`KeyBinding`, `PopupWindow`, `Flickable`, `TextInput`, `Path`, and
`VirtualWindow`. `Pressable`, `PopoverPanel`, `SwitchControl`, and `VList`
were removed from the schema. Rust continues to own input routing, focus,
portal hosting, text editing, rendering, and virtual-window calculation.

The standard library contains separate `.argui` components for `Button`,
`Switch`, `Slider`, `Input`, `TextArea`, `Popover`, `Dialog`, `Menu`,
`MenuItem`, `Select`, `Combobox`, `ScrollView`, and `ListView`. The gallery
uses those components, including `ListView` for its navigation sidebar.
The DSL expresses their visual states and interactions through primitive
properties, event handlers, child references, and theme tokens.

Parser, semantic checks, typed IR, AOT generation, and live evaluation now
carry observed pointer/focus/scroll state, typed event payloads, authored
paths, and generic effect definitions. Targeted tests cover both execution
paths, component interaction, invalid input, and hot-reload invalidation.
The final release, platform, coverage, and quality gates are tracked in the
[delivery checklist](README.md).

# Generic visual effects

An `effect` applies an imported WGSL shader to any visual element. The
optional literal `scope` selects `"whole"` (default), `"background"`,
`"border"`, `"content"`, `"text"`, or `"backdrop"`. The first five scopes
operate on that element's paint; `"backdrop"` operates on pixels behind it.
Parameters remain typed and reactive in both generated and live builds.

```argui
export effect EdgeFire {
    shader: "assets/edge-fire.wgsl"
    parameter intensity: float = 0.8
}

Rectangle {
    background: solid(#ffffffb8)
    backdrop_filter: "blur(14px)"
    border_color: #ffffff
    border_width: 2.0
    effect: EdgeFire { scope: "border" intensity: 0.9 }
}
```

`backdrop_filter` is a common visual property on `Rectangle`, `Text`, layout
containers, images, vector paths, text inputs, and portal surfaces. It samples
pixels already painted behind the element, including content outside its
bounds needed by a blur kernel. The result is clipped to the element bounds
and its rounded mask when present. On `Text`, the blur covers its laid-out box,
not individual glyph outlines. A translucent background lets the filtered
scene show through.

The shader is attached to the `Rectangle` that paints the surface. A custom
button can place the same rectangle inside `TouchArea`; a custom popover can
place it inside `PopupWindow`. The standard Popover forwards translucent fill
and numeric blur to its surface rectangle. The Liquid glass gallery page has
contrasting bands behind both a `Rectangle` and a `Text` surface so the filter
can be compared at zero and nonzero radius.
