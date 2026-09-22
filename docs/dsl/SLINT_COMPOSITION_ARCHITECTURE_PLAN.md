# Argui DSL: Slint-style composition architecture

> **Status: implementation in progress; final quality and platform gates remain.**
> This plan complements the existing [DSL implementation plan](ARGUI_DSL_IMPLEMENTATION_PLAN.md)
> and [delivery checklist](README.md). The implemented primitive boundary is
> recorded in the [composition inventory](composition-inventory.md).

## Mission and architectural rule

Refactor Argui so complex controls are authored in `.argui` from fundamental
elements. Follow the **same architectural separation as Slint**: Rust supplies
the renderer, platform integration, input routing, focus, accessibility, models,
and other low-level mechanisms; the UI language composes them into controls.
This is architectural parity, not Slint source-code reuse or a requirement to
copy its grammar.

The decisive proof is a custom Button built from `Rectangle`, `Text`, and
`TouchArea`, and a draggable Slider built from `TouchArea` pointer coordinates,
as in [Slint's custom-control examples](https://docs.slint.dev/latest/docs/slint/guide/development/custom-controls/).
Neither control may require a new native Rust widget type. Argui's existing
language capabilities must be retained while the missing low-level capabilities
are added.

## Required primitive contract

These names and responsibilities are deliberate. Do not substitute a vague
"generic widget" whose Rust implementation still makes Button-, Menu-, or
ListView-specific decisions.

| Primitive | Required responsibility | Must not own |
| --- | --- | --- |
| `Rectangle` | Fill/brush, border, radius, clip, opacity, transforms, and generic visual effects. See [Slint Rectangle](https://docs.slint.dev/latest/docs/slint/reference/elements/rectangle/). | Button/Card state, variant, or behavior. |
| `TouchArea` | Hover/pressed state, pointer positions, press/release/click/move/wheel, pointer capture, touch events, and event acceptance. See [Slint TouchArea](https://docs.slint.dev/latest/docs/slint/reference/gestures/toucharea/). | Colors, hover paint, button role, or a predefined control style. |
| `FocusScope` and `KeyBinding` | Programmatic focus, focus state, typed key events and modifiers, keyboard shortcuts, propagation, and modal focus handling. See [Slint FocusScope](https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/focusscope/). | The layout or appearance of an Input, Menu, or Dialog. |
| `PopupWindow` | Portal/overlay lifecycle, anchor or viewport placement, edge avoidance, close policy, and focus transfer/restoration. See [Slint PopupWindow](https://docs.slint.dev/latest/docs/slint/reference/window/popupwindow/). | Popover/Menu/Dialog style or one hardcoded placement policy. |
| `Flickable` | Viewport, content extent, scrolling position, and scroll input/physics. See [Slint Flickable](https://docs.slint.dev/latest/docs/slint/reference/gestures/flickable/). | A mandatory scrollbar, edge shadow, or ListView visual design. |
| `TextInput` | Editable text, IME, caret, selection, editing commands, and low-level text events. See [Slint TextInput](https://docs.slint.dev/latest/docs/slint/reference/keyboard-input/textinput/). | Label, field frame, placeholder style, validation UI, or automatic scrolling policy. |
| `Text`, `Image`, `Svg`, `Path` | Composable text, raster/vector assets, and path geometry. See [Slint Path](https://docs.slint.dev/latest/docs/slint/reference/elements/path/). | A restriction to predefined widget shapes or a Tabler-only asset path. |

All visual elements need the applicable common properties: geometry, visibility,
transforms, opacity, animation, clipping, semantics, and accessibility. Implement
these as shared capabilities rather than per-widget options. Pointer and focus
state must be readable in reactive DSL expressions, such as a Rectangle brush
depending on `touch.pressed` or `touch.has-hover`. See [Slint common
properties](https://docs.slint.dev/latest/docs/slint/reference/common/).

For scrolling, keep native virtualization/windowing when necessary for
performance, but expose its data/window contract without dictating the visual
scrollbar or edge shadows. A DSL `ListView` must instantiate only visible rows,
as [Slint ListView](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/)
does.

## Exact Rust/DSL boundary

Rust owns the OS window backend, rendering, hit testing, event dispatch,
pointer capture, focus engine, keyboard/IME bridge, accessibility bridge,
portal hosting, asset loading, model infrastructure, virtualized windowing,
and application-provided business functions. These mechanisms must stay
independent of the DSL implementation.

The DSL owns widget structure, visual states, styles, themes, interactions made
by composing those mechanisms, and the standard widgets. Each of `Button`,
`Switch`, `Slider`, `Input`, `TextArea`, `ScrollView`, `ListView`, `Select`,
`Combobox`, `Popover`, `Menu`, `Dialog`, and related controls should have a
separate `.argui` implementation. Their existence must not require Rust
dispatch by widget name. This follows the separation between Slint's base
elements and [standard widgets](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/overview/).

Audit the current Argui paths before replacing them. In particular, a DSL
`Button` that delegates hover/pressed painting to native `Pressable` properties
is not complete. A native `PopoverPanel` with fixed placement/dismiss behavior
and a native `VList` that dictates scrollbar/edge-shadow visuals are not the
final primitives. Extract reusable mechanisms and migrate callers; do not
maintain two permanent implementations.

## Language and execution requirements

Carry each capability through syntax, schema, semantic checking, typed IR,
AOT code generation, live runtime, and hot reload:

- Typed properties, derived values, reactive bindings, and two-way editing
  where appropriate.
- Typed callback/event payloads, propagation and event handling, pointer
  coordinates, and focus/keyboard events.
- Child references, property forwarding/aliases, slots, composition,
  conditionals, repeaters, and dynamic models.
- Component-local state with stable identity across unrelated live edits;
  preserve focus, selection, scroll position, and open overlays when valid.
- Generic animation and effects on every applicable visual element, nested
  component, SVG, and image. Do not add a special animation path for a named
  widget or hardcode a Liquid Glass effect.
- Shared semantic theme tokens for light, dark, and custom themes, without
  duplicating color logic inside each component.
- Composable accessibility role, name, state, action, and keyboard navigation.

The AOT and live paths must produce equivalent behavior. A successfully
applied hot-reload generation must visibly update without a click or unrelated
input. Diagnostics must identify source locations and explain rejected edits.
No compiler, IR, or runtime branch may depend on a standard widget's name.

## Required proof components

Create separate `.argui` files and usable widget-gallery pages for these
components, with mouse/touch/keyboard behavior, light/dark/custom themes,
accessibility, AOT/live parity, and tests:

1. **Button:** `Rectangle + Text + TouchArea`, with DSL-owned hover, press,
   focus, disabled, busy, and variant styling. All variants visibly react.
2. **Slider:** draggable thumb and value calculation from `TouchArea` pointer
   coordinates, without `NativeSlider` or native Slider-specific behavior.
3. **Input and TextArea:** compose `TextInput` with DSL labels, frame,
   placeholder, validation, and reusable caret/selection customization.
4. **Dialog, Popover, Menu/submenu, Select, Combobox:** compose `PopupWindow`,
   `FocusScope`, `KeyBinding`, `TouchArea`, and visual elements. Test focus
   restoration, keyboard navigation, outside click, and window-edge placement.
5. **ScrollView and virtualized ListView:** compose `Flickable` and the native
   virtual-window mechanism; draw custom scrollbar and optional edge shadows
   in DSL. Exercise the gallery's responsive sidebar with this same base.
6. **Original complex visual:** combine `Path`, imported SVG/image, gradients,
   generic effects, and animation to prove the language is not limited to
   conventional controls.

The gallery must allow the user to interact with every proof component. A
static screenshot or a small hardcoded native demo is insufficient.

## Delivery phases and acceptance

- [ ] **1. Inventory and contracts.** Map current native widgets, specialized
  branches, DSL features, and tests. Record the primitive APIs and migration
  order before implementation. Preserve working behavior.
- [ ] **2. Fundamental primitives.** Extract/implement the elements above,
  shared properties, input/focus contracts, and bounded invalidation. Reuse
  existing engine machinery where suitable, without duplicating systems.
- [ ] **3. Language pipeline.** Implement every missing feature end to end
  through parser, semantic checks, IR, compiler, runtime, live protocol, and
  diagnostics. Test each stage and invalid-input behavior.
- [ ] **4. Button and Slider proof.** Complete both without specialized native
  widgets. If they need a special-case branch, improve the underlying
  primitive or DSL before advancing.
- [ ] **5. Text, overlays, and models.** Complete Input/TextArea, Dialog,
  Popover, Menu, Select/Combobox, ScrollView/ListView, and visual proof.
- [ ] **6. Migration and removal.** Replace old high-level native paths and
  callers, remove obsolete code and dependencies, update docs and examples.
  Do not leave placeholders or permanent compatibility facades.
- [ ] **7. Final verification.** Run targeted `cargo nextest run -p PACKAGE
  --all-features` during development, then check release and AOT/live parity,
  hot reload without user input, cross-platform build paths, keyboard and
  accessibility behavior, idle work, virtualization cost, and bounded effect
  damage. Run GUI checks through the private Linux display and inspect saved
  captures; a blank capture is a failure. Every per-crate and workspace
  coverage metric must be at least 85%. Run `./scripts/quality.sh` exactly once
  after implementation and immediately before the final commit, following
  `AGENTS.md` and `docs/contributing/code-quality.md`.

## Multi-agent execution

Assign one subagent per major workstream with non-overlapping file ownership
and explicit contracts: interaction/focus primitives; visuals/assets/effects;
language/compiler/IR; overlays/scroll/models; DSL standard library/gallery.
Use waves if concurrency is limited. The coordinator integrates interfaces
and alone runs Cargo compilation, whole-workspace tests, and LLVM coverage to
avoid lock contention and competing instrumented artifacts. Subagents may
write tests but must not compile concurrently.

Do not stop after scaffolding or a few sample widgets. When a proof component
cannot be completed, implement the missing reusable primitive, then finish the
component and its tests. Report genuine blockers honestly instead of labeling
a Rust-widget facade as a DSL implementation.

## Explicit non-solutions

- No `NativeSlider`, native Button styling engine, or `Pressable` variant
  properties standing in for DSL bindings.
- No fixed-style `PopoverPanel`, Menu, Dialog, or Select primitive.
- No scrollbar or edge shadows hardwired into `Flickable` or virtualization.
- No `if component_name == "Button"`-style compiler/runtime dispatch.
- No widget-specific animation, gradient, shader, or asset pipeline.
- No AOT/live behavior divergence, inert hot reload, placeholders, or
  unremoved replacement branches.

The decisive exit test remains: a developer can build and style a new Button
using `Rectangle + Text + TouchArea`, with hover, pressed, focus, and disabled
states expressed in `.argui`, without changing Rust.
