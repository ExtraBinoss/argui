# Argui DSL — Architecture, Core Refactors, Live Development, AOT Release, and Implementation Plan

> **Audience:** AI coding agents and maintainers implementing the Argui declarative UI language.
>
> **Status:** Architecture/implementation plan. This document is intentionally prescriptive.
>
> **Primary product decision:** `.argui` becomes the **default and primary way to author UI**. Rust remains the implementation language of the engine and the application/business-logic language. Ordinary application authors should not need to write `Element`, `Context`, `StylePatch`, `MotionBinding`, widget builders, or other Argui Rust UI code.

---

## 0. Executive decision

Argui should become a **language-first native UI framework**:

```text
Application UI       Application logic          GPU effects
     .argui                 .rs                     .wgsl
        │                    │                        │
        └──────────────┬─────┴────────────────────────┘
                       │
                       ▼
                Argui toolchain
                       │
             ┌─────────┴─────────┐
             │                   │
           DEV                 RELEASE
             │                   │
             ▼                   ▼
       live compiled IR       Rust AOT codegen
       + DSL runtime              │
             │                    │
             └─────────┬──────────┘
                       ▼
                 Argui engine
              UiTree / layout /
             animation / render
                       │
                       ▼
                      WGPU
```

The required split is:

- **Development:** `.argui` is incrementally parsed, type-checked, compiled to typed IR, sent to a live DSL runtime, and atomically reloaded without recompiling Rust.
- **Release:** the exact same typed IR semantics are lowered to generated Rust at build time. The final application does **not** link the parser, compiler, live runtime, file watcher, LSP, or DSL interpreter.
- **WGSL:** custom shaders remain real `.wgsl` files. Do not invent a shader DSL.
- **Rust:** user Rust code owns business logic, services, filesystem, database, networking, async work, platform-specific integrations, and explicit native extension points. It should not be required for normal UI composition or styling.

The architecture must make it impossible for the live path and release path to silently acquire different semantics.

---

# 1. Product goals

## 1.1 User-facing goal

A normal Argui application should look like:

```text
my-app/
├── Cargo.toml
├── build.rs
├── src/
│   ├── main.rs
│   └── app/
│       ├── project.rs
│       ├── storage.rs
│       └── services.rs
├── ui/
│   ├── main.argui
│   ├── theme.argui
│   ├── styles.argui
│   ├── models.argui
│   └── components/
│       ├── sidebar.argui
│       ├── project-card.argui
│       └── editor.argui
├── shaders/
│   ├── glow.wgsl
│   └── glass.wgsl
└── assets/
    ├── logo.svg
    └── splash.png
```

The author should be able to implement nearly all presentation-layer concerns in `.argui`:

- component trees;
- reusable components;
- component imports/re-exports;
- typed properties;
- callbacks;
- two-way bindings;
- local UI state;
- conditions;
- repeaters/models;
- slots/children;
- layout;
- styles;
- theme tokens;
- runtime theme switching;
- visual states;
- transitions;
- keyframes;
- tweens;
- springs;
- accessibility properties;
- images/SVGs;
- native Argui widgets/components;
- custom WGSL effects;
- responsive/container-query behavior;
- localization bindings;
- overlays/popups/dialog composition.

Rust should remain necessary for application logic, not ordinary UI authoring.

## 1.2 Development goal

`argui dev` should provide:

- transactional hot reload of `.argui`;
- transactional hot reload of `.wgsl`;
- preservation of application state;
- preservation of declarative component state;
- preservation of Argui retained UI state (focus, selection, scrolling, text editing, open overlays where identity survives);
- preservation/retargeting of compatible animations;
- previous valid UI remains mounted if the new source fails to parse/type-check/validate;
- structured diagnostics with exact source spans;
- incremental compilation of affected modules only;
- usable native, WebAssembly, Android, and iOS development transports without putting the compiler in release binaries.

## 1.3 Release goal

A release build must contain:

- generated Rust UI code;
- the normal Argui engine/runtime pieces that the generated UI needs;
- application Rust logic;
- reachable assets;
- reachable WGSL.

A release build must **not** contain:

- DSL lexer/parser;
- CST/AST/HIR compiler structures;
- semantic database;
- DSL bytecode/expression interpreter;
- live reload protocol/server;
- file watcher;
- LSP;
- CLI;
- source files unless explicitly requested;
- development source maps unless explicitly requested.

This should be validated in CI using `cargo tree`/fixture applications.

---

# 2. Non-goals

Do **not** turn Argui DSL into any of the following:

- JavaScript/JSX;
- HTML;
- CSS;
- Lua used as the primary UI language;
- Python;
- embedded Rust syntax;
- a general-purpose scripting language;
- a shader language;
- a browser/DOM compatibility layer.

Familiar syntax is good. Compatibility promises with unrelated ecosystems are not.

The DSL should be deliberately constrained so that the compiler always knows:

- the type of every property;
- the dependencies of every binding;
- the identity of every definition/site;
- what can be preserved across reload;
- which update phase a changed value can require;
- the public ABI between UI and Rust.

---

# 3. Repository rules the implementation agent MUST follow

The current Argui repository has explicit contributor rules. Preserve them.

## 3.1 Source organization

- Keep every `.rs` file at **600 physical lines or fewer**.
- Split by responsibility, not just because a file approached 600 lines.
- Do not dump an entire compiler/runtime into `src/lib.rs`.
- Tests belong in `tests/`, mirroring paths under `src/`.
- Test code is forbidden under `src/`.
- Add dependencies only to the crate that consumes them.
- Renderer/runtime/core engine crates must never depend on the DSL crates.
- Every public function/method needs accurate Rustdoc.
- Do not commit `TODO`, `FIXME`, `todo!`, `unimplemented!`, or placeholder implementations.
- Preserve the current quality requirement: 85% coverage floor for lines/functions/regions/branches.
- Run targeted tests during work and the repository quality script once at the final gate.

## 3.2 Dependency direction

Keep the fundamental direction:

```text
applications
    │
DSL build/live integration + argui facade
    │
DSL compiler/runtime adapters
    │
runtime + widgets/native primitives
    │
ui + layout + renderer
    │
core + paint + text + animation + accessibility
```

Never create:

```text
argui-ui     -> argui-dsl-*
argui-render -> argui-dsl-*
argui-runtime-> argui-dsl-*
```

The engine must work without the DSL.

---

# 4. Current Argui foundations that should be kept

Do not rewrite strong existing systems just because a DSL is being added.

## 4.1 Keep `UiTree` and retained reconciliation

Current Argui already has:

- retained `Element` trees;
- stable `NodeId`s;
- structural reconciliation;
- explicit `Element::keyed`;
- retained focus;
- retained scrolling;
- retained text-input state;
- retained selection;
- retained gestures/interaction;
- incremental update classification.

This is the foundation of DSL hot reload.

## 4.2 Keep update classification

The current distinction between:

```text
Semantics
Composite
Paint
Scroll
Layout
```

is extremely valuable.

DSL bindings, theme changes, transitions, and animations should eventually lower to the smallest correct update class.

## 4.3 Keep the animation engine

`argui-animation` already provides strong primitives:

- `Motion`;
- `MotionBinding`;
- `Tween`;
- easing;
- keyframes;
- timelines;
- springs;
- decay/inertia;
- transitions;
- composition.

Do **not** replace it with a DSL-specific animation engine.

The missing piece is DSL-level stable animation identity/state ownership, which belongs above the animation engine.

## 4.4 Keep `StyleTransition`

The retained transition registry keyed by retained UI identity is useful and should stay.

## 4.5 Keep WGPU renderer/layout/text/accessibility

The DSL is an authoring/runtime layer above these systems.

No DSL logic belongs in:

- `argui-render`;
- `argui-layout`;
- text shaping;
- AccessKit/native accessibility;
- platform event translation.

## 4.6 Keep real WGSL and Naga validation

Current Argui already validates custom WGSL and has a typed custom effect parameter ABI.

Preserve this. Improve its dynamic/reload boundary; do not replace WGSL.

---

# 5. What is currently incompatible with a clean live DSL

This section describes blockers that should be solved **before the public DSL syntax is frozen**.

---

## 5.1 Blocker: static-string identities

Several current APIs assume names are Rust literals with `'static` lifetime. Examples include concepts such as:

- `StateName(&'static str)`;
- `StateScopeId(&'static str)`;
- `ContainerScopeId(&'static str)`;
- `ActionId(&'static str)`;
- `EffectId(&'static str)`;
- effect parameter names;
- effect pass names;
- effect-target parameter names;
- scroll-effect parameter names.

A live compiler receives owned file content, not `'static` literals.

### Required change

Introduce a shared engine-level name type with these properties:

- accepts both static and dynamically loaded names;
- never requires `Box::leak`;
- cheap clone;
- cheap equality/hash;
- no process-global mutable registry requirement;
- usable in core/UI/paint/render without importing DSL crates.

Recommended initial shape:

```rust
pub struct Name(Arc<str>);
```

with constructors similar to:

```rust
Name::from_static("hovered")
Name::from_owned(String::from("hovered"))
```

A DSL runtime may additionally intern names so repeated values share one `Arc`.

Do not prematurely encode compiler `SymbolId` directly into core engine APIs. Compiler symbol IDs are compiler-database identities. Engine names are runtime identities. They are related but should not be the same abstraction.

### Convert at minimum

- `StateName`;
- `StateScopeId`;
- `ContainerScopeId`;
- `ActionId`;
- `EffectId`;
- `EffectArgument.name`;
- `EffectParameter.name`;
- `EffectPassDefinition.name`;
- any property/effect target that stores a parameter `&'static str`.

### Acceptance

- no DSL-loaded identifier requires a leaked allocation;
- existing Rust static-name constructors remain ergonomic;
- no dependency from engine crates to DSL crates;
- equality/hash tests cover static/dynamic names.

---

## 5.2 Blocker: element identity is not strong enough for source edits

Current reconciliation can preserve keyed children and compatible position-based children. A source-level DSL needs a stronger identity model so inserting a sibling does not unnecessarily destroy another stateful element.

Example:

```text
before:
Input
Button

after:
Text
Input
Button
```

The DSL compiler already knows these are the same `Input` and `Button` source sites.

### Required change

Add a **private retained identity channel** to `Element` that is distinct from the user-facing key/focus key.

Conceptually:

```rust
pub struct RetainedIdentity(...);
```

and:

```rust
Element::retained_identity(identity)
```

The DSL can build identities from:

- stable compiled `SiteId`;
- component instance identity;
- optional repeater runtime key.

Do not overload the existing public string key for compiler identity.

### Identity rules

Static source node:

```text
ComponentDefinitionId + SiteId
```

Repeated node:

```text
ComponentInstanceId + SiteId + RepeaterKey
```

Conditional node:

- same site keeps identity if the branch is the same semantic site;
- different source sites do not accidentally inherit each other's retained state.

### Acceptance

Tests must prove that inserting/reordering unrelated source sites preserves:

- input focus;
- input selection;
- text-editing state;
- scroll offsets;
- open state when ownership remains;
- transition continuity where applicable.

---

## 5.3 Blocker: there is no language-independent reactive property core

The DSL requires properties and bindings such as:

```text
property count: int = 0
property doubled: int = count * 2

Text {
    text: "Count: " + count
}
```

A correct implementation needs dependency tracking, transactions, and cycle detection.

### Recommended core addition

Create a language-independent crate:

```text
crates/argui-reactive/
```

This is **not** a DSL crate. It is a typed reactive primitive layer usable by generated Rust and by the live DSL runtime.

Suggested responsibilities:

```text
Property<T>
Read tracking
Binding dependency registration
Observers
Revision/dirty tracking
Transactions
Cycle detection
Two-way-link primitive
Batched notification
```

Do not put ASTs, expression bytecode, module names, or DSL values in this crate.

### Why shared reactive core matters

Development path:

```text
Dynamic DSL value
  -> argui-reactive dependency graph
```

Release path:

```text
generated Property<T>
  -> same argui-reactive dependency graph
```

This prevents two unrelated reactivity implementations from drifting semantically.

### Acceptance

- bindings update only when dependencies change;
- an idle UI performs no work;
- cycles produce deterministic errors;
- batched changes do not cause repeated redundant view work;
- two-way bindings cannot infinite-loop;
- generated typed properties and dynamic DSL properties can share observer semantics.

---

## 5.4 Blocker: callback slots are a Rust render implementation detail

Current local event handlers are presentation-owned slots allocated during Rust rendering. That is valid internally but not a stable DSL-to-business-logic ABI.

The DSL needs:

```text
callback save(path: string)
```

to mean the same thing before and after a source reload.

### Required architecture

Separate:

1. **local routed UI event handler identity** — internal runtime concern;
2. **declarative component callback identity** — public component ABI.

Create a DSL/runtime-neutral public callback boundary based on generated component API, not render slot order.

Compiler concept:

```text
ComponentCallbackId
```

Generated Rust exposes typed callback binding methods/traits.

Live runtime maps `ComponentCallbackId` to a Rust host binding created when the root component is connected.

### Acceptance

- editing unrelated event handlers cannot remap `save` to another callback;
- callback argument/return types are validated at compile time;
- public callback signature changes are detected as ABI changes during live reload.

---

## 5.5 Blocker: theme values are too narrow

Current `ThemeValue` is essentially:

```text
Color
Number(f32)
```

The intended DSL theme system requires strongly typed tokens.

### Required Theme v2

Support token value categories such as:

```text
color
brush/fill
float
int
bool
length
dimension where appropriate
percentage
duration
angle
radius/corner-radii
insets
border
shadow
font family
font weight
font size
line height
transform
```

Avoid one untyped `Number` for everything.

### Required token definition model

A runtime theme should know:

```text
ThemeTokenDefinition {
    key
    type
    default
}
```

and a theme instance stores typed overrides.

The DSL semantic compiler must reject:

```text
padding: var(--background)
```

when `--background` is a color.

### Required dependency tracking

Theme reads should become observable dependencies so changing one token does not necessarily invalidate every theme consumer.

Conceptually:

```rust
cx.theme(token_key)
```

registers a dependency.

### Acceptance

- runtime theme changes work without application restart;
- theme tokens are typed;
- token dependency cycles are rejected;
- changing a paint-only token does not force unrelated layout work;
- theme changes can be observed by all relevant mounted components.

---

## 5.6 Blocker: widget Rust APIs are not a declarative schema

The DSL compiler/LSP cannot be a giant handwritten `match` over every Rust widget.

### Long-term design

Create a lightweight engine-facing declarative schema crate:

```text
crates/argui-schema/
```

It must not depend on the DSL.

Responsibilities:

```text
ValueType schema
Native element/component schema
Property schema
Event schema
Slot schema
Variant schema
Style-part schema
Documentation metadata
```

### Important product direction

Do **not** make every current Rust widget a permanent DSL primitive.

Prefer:

```text
small native primitive/behavior surface in Rust
             +
official @argui/ui components authored in .argui
```

This allows Argui's own public widgets to dogfood the DSL.

Examples of native behavior primitives that may stay Rust-backed:

- pointer/press interaction;
- text editing;
- scrolling;
- virtual-list engine;
- overlay positioning;
- menu/focus behavior;
- range/slider interaction;
- color picking internals;
- native platform integrations.

Visual composition, theme mapping, and ordinary component structure should migrate to the `.argui` standard library when practical.

### Acceptance

The same canonical schema information feeds:

- semantic compiler;
- live adapters;
- LSP completion/hover;
- generated documentation;
- AI schema queries.

---

## 5.7 Blocker: theme and widget appearance are too coupled to `WidgetTheme`

Current widgets frequently build concrete Rust style objects from `WidgetTheme`.

This is useful today but prevents the desired theme/style model from being the public source of truth.

### Required migration direction

Split:

```text
behavior
appearance
```

Behavior remains native where necessary.

Appearance should increasingly be defined by:

```text
theme tokens
named styles
component styles
variants
states
```

in DSL standard-library components.

Keep compatibility internally during migration, but do not design the DSL around the shape of `WidgetTheme`.

---

## 5.8 Blocker: effect definitions are static/immutable-oriented

Current custom effects have excellent foundations but definitions use static names/source slices and a registry designed around startup registration.

### Required Effect Registry v2

Effect definitions must be owned/dynamic and revision-aware.

Conceptually:

```text
EffectDefinition {
    id: EffectId
    revision: u64
    parameters: Arc<[EffectParameter]>
    passes: Arc<[EffectPassDefinition]>
}
```

A development API must support transactional replacement:

```text
prepare replacement
validate all passes
create GPU pipelines
swap if all succeeded
keep old version on failure
```

Do not destroy a functioning effect because a developer saved invalid WGSL.

### Acceptance

- valid `.wgsl` edit replaces the live pipeline;
- invalid `.wgsl` edit leaves the previous pipeline active;
- schema changes trigger correct buffer/layout handling;
- removed effects are released when no longer reachable.

---

## 5.9 Blocker: WGSL validation is too tied to `argui-render`

The DSL build compiler should validate WGSL without needing to drag the whole WGPU renderer into the compiler dependency graph.

### Recommended extraction

Create:

```text
crates/argui-shader/
```

Responsibilities:

- Argui custom-effect ABI source wrapping;
- Naga parse/validation;
- effect entry-point contract checks;
- user-source span mapping through generated ABI header/footer;
- effect parameter metadata validation;
- source hashing.

`argui-render` uses this crate.
`argui-dsl-compiler` uses this crate.

The shader crate must not depend on WGPU.

### Acceptance

Build-time compiler can report exact `.wgsl` diagnostics without constructing a renderer.

---

## 5.10 Blocker: shader diagnostics need source mapping

Argui prepends/appends ABI WGSL around user source.

Diagnostics from Naga must map back to:

```text
shaders/glow.wgsl:line:column
```

not wrapped-source line numbers.

Implement an explicit `ShaderSourceMap`.

---

## 5.11 Blocker: custom-effect parameter storage must survive dev schema growth

A live shader edit may add parameters.

Do not permanently size GPU parameter storage based only on startup definitions if live definitions can grow.

Required behavior:

- compute required max parameter words after accepted definition updates;
- grow/recreate GPU parameter storage when necessary;
- never shrink eagerly;
- preserve idle behavior.

---

## 5.12 Blocker: asset IDs are process-fresh rather than source-stable

Current image/vector helpers allocate fresh numeric IDs.

For hot-reloaded source assets, use a stable asset identity plus revision.

Concept:

```text
AssetKey
AssetRevision
```

`AssetKey` is stable for a canonical imported asset.
`AssetRevision` changes when its contents change.

Example:

```text
assets/logo.svg
same AssetKey
new revision after edit
```

The renderer updates the resource instead of forcing every DSL reference to receive a completely unrelated identity.

Do not replace all existing `ImageId`/`VectorId` semantics unnecessarily. Add an asset registry layer that owns stable source-to-runtime handles.

---

## 5.13 Missing: DSL-aware debug metadata for DevTools

Development inspection should be able to show:

```text
component: ProjectCard
source: ui/components/project-card.argui:42
site: ProjectCard.card_root
styles:
  CardStyle
theme reads:
  --surface
  --radius-md
```

Add optional development metadata at the `Element`/inspection boundary.

This metadata must be stripped or minimized in release by default.

---

# 6. Core changes that are NOT required

Avoid unnecessary rewrites.

## 6.1 Do not replace `Motion` pointer identity in the engine merely for DSL

The live DSL runtime can keep the same `Motion<T>` object alive in an `AnimationStore` keyed by DSL identity. If the same handle is reused after reload, current motion identity remains valid.

Therefore:

- keep `argui-animation` engine identity behavior unless a concrete missing capability appears;
- implement DSL animation persistence in `argui-dsl-runtime`.

## 6.2 Do not move DSL module concepts into `argui-runtime`

`FileId`, AST definitions, imports, compiler symbols, etc. remain DSL-toolchain concerns.

## 6.3 Do not embed an interpreter into `argui-ui`

The live interpreter belongs in `argui-dsl-runtime`.

---

# 7. Target repository layout

Add the DSL family as a top-level grouped workspace area.

Change root workspace membership from only:

```toml
members = ["crates/*"]
```

to include:

```toml
members = [
    "crates/*",
    "argui-dsl/*",
]
```

Recommended layout:

```text
argui-dsl/
├── README.md
│
├── dsl-syntax/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── kind.rs
│   │   ├── text.rs
│   │   ├── tree/
│   │   └── ast/
│   └── tests/
│       ├── tree/
│       └── ast/
│
├── dsl-parser/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lexer/
│   │   ├── grammar/
│   │   │   ├── declaration.rs
│   │   │   ├── component.rs
│   │   │   ├── expression.rs
│   │   │   ├── style.rs
│   │   │   ├── theme.rs
│   │   │   ├── animation.rs
│   │   │   └── effect.rs
│   │   └── recovery/
│   └── tests/
│       ├── lexer/
│       ├── grammar/
│       └── recovery/
│
├── dsl-semantic/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── db/
│   │   ├── module/
│   │   ├── import/
│   │   ├── resolve/
│   │   ├── types/
│   │   ├── component/
│   │   ├── property/
│   │   ├── binding/
│   │   ├── style/
│   │   ├── theme/
│   │   ├── animation/
│   │   ├── effect/
│   │   └── diagnostics/
│   └── tests/
│       └── ... mirrors src responsibilities ...
│
├── dsl-ir/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── module/
│   │   ├── component/
│   │   ├── expression/
│   │   ├── property/
│   │   ├── style/
│   │   ├── theme/
│   │   ├── animation/
│   │   ├── effect/
│   │   ├── asset/
│   │   └── debug/
│   └── tests/
│
├── dsl-compiler/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lower/
│   │   ├── optimize/
│   │   ├── reachability/
│   │   ├── rust_codegen/
│   │   │   ├── module.rs
│   │   │   ├── component.rs
│   │   │   ├── property.rs
│   │   │   ├── expression.rs
│   │   │   ├── style.rs
│   │   │   ├── animation.rs
│   │   │   ├── effect.rs
│   │   │   └── asset.rs
│   │   ├── wgsl/
│   │   └── output/
│   └── tests/
│       ├── lower/
│       ├── codegen/
│       └── fixtures/
│
├── dsl-runtime/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── module/
│   │   ├── component/
│   │   ├── property/
│   │   ├── expression/
│   │   ├── event/
│   │   ├── model/
│   │   ├── identity/
│   │   ├── animation/
│   │   ├── theme/
│   │   ├── style/
│   │   ├── effect/
│   │   ├── asset/
│   │   ├── migration/
│   │   ├── reload/
│   │   └── inspect/
│   └── tests/
│       └── ... mirrors src responsibilities ...
│
├── dsl-protocol/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── version.rs
│   │   ├── message/
│   │   └── transport/
│   └── tests/
│
├── dsl-build/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   └── cargo.rs
│   └── tests/
│
├── dsl-cli/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── command/
│   │   │   ├── check.rs
│   │   │   ├── dev.rs
│   │   │   ├── fmt.rs
│   │   │   ├── preview.rs
│   │   │   ├── schema.rs
│   │   │   └── screenshot.rs
│   │   └── server/
│   └── tests/
│
├── dsl-lsp/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── server/
│   │   ├── diagnostics/
│   │   ├── completion/
│   │   ├── hover/
│   │   ├── navigation/
│   │   ├── references/
│   │   ├── rename/
│   │   ├── symbols/
│   │   ├── semantic_tokens/
│   │   └── formatting/
│   └── tests/
│
├── dsl-testing/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fixture/
│   │   ├── parity/
│   │   └── snapshot/
│   └── tests/
│
└── stdlib/
    ├── package.argui
    ├── theme/
    ├── styles/
    └── ui/
        ├── button.argui
        ├── input.argui
        ├── card.argui
        └── ...
```

Crate package names should be:

```text
argui-dsl-syntax
argui-dsl-parser
argui-dsl-semantic
argui-dsl-ir
argui-dsl-compiler
argui-dsl-runtime
argui-dsl-protocol
argui-dsl-build
argui-dsl-cli
argui-dsl-lsp
argui-dsl-testing
```

Do not create a separate `hot-reload` crate unless a real responsibility boundary later justifies it. Reload is a collaboration between:

- compiler/dev server;
- protocol;
- live runtime.

---

# 8. Compiler architecture

Use one semantic frontend.

```text
source
  │
  ▼
lossless CST
  │
  ▼
typed AST facade
  │
  ▼
module/import resolution
  │
  ▼
HIR / semantic model
  │
  ▼
type checking + binding analysis
  │
  ▼
typed IR
  │
  ├──────────────► Rust AOT backend
  │
  └──────────────► Live runtime package
```

## 8.1 Lossless CST is required

Do not implement a parser that only returns a valid AST or an error.

The syntax tree must survive:

- incomplete code;
- invalid tokens;
- comments;
- whitespace;
- missing delimiters;
- partially typed expressions.

This is required for:

- LSP;
- formatting;
- source-preserving refactors;
- good error recovery.

A `rowan`-style green tree architecture is a good fit.

## 8.2 Every syntax/semantic node needs a span

Use:

```rust
Span {
    file: FileId,
    range: TextRange,
}
```

Track spans for:

- definitions;
- imports;
- expressions;
- property declarations;
- property assignments;
- styles;
- theme tokens;
- animation clauses;
- callback declarations/calls;
- effect definitions;
- WGSL file references;
- asset references.

## 8.3 Incremental semantic database

The compiler and LSP should share an incremental query database.

Recommended query categories:

```text
parse(FileId)
module_for(FileId)
module_exports(ModuleId)
resolve_import(ImportId)
definition(SymbolRef)
type_of(ExpressionId)
expected_type(ExpressionId)
binding_dependencies(PropertyId)
component_schema(ComponentId)
theme_schema(ThemeId)
effect_schema(EffectId)
lower_component(ComponentId)
public_api_hash(ComponentId)
reachable_from(RootComponent)
```

Use an incremental engine such as Salsa or an equivalently disciplined internal query system.

Do not make the LSP own its own semantic cache.

---

# 9. Type system

Freeze the type system before freezing syntax details.

## 9.1 Core scalar/value types

At minimum:

```text
bool
int
float
string

length
percentage
dimension
duration
angle

color
brush

point
size
rect
transform

insets
radii
border
shadow
```

`dimension` is intentionally not just `length`; it can represent values such as:

```text
auto
fill
42px
50%
```

## 9.2 Collection/domain types

```text
optional<T>
array<T>
model<T>
```

`model<T>` is observable and supports incremental list mutation.

## 9.3 User-defined types

```text
struct
enum
```

Example:

```text
export struct Project {
    id: string
    name: string
}

export enum Status {
    idle
    loading
    failed
}
```

Public structs/enums used by Rust bindings require generated Rust equivalents/conversions.

## 9.4 Function/callback types

Callbacks are typed.

Example:

```text
callback save(path: string)
callback confirm() -> bool
```

If asynchronous Rust actions are needed, model them explicitly rather than making all DSL functions async. Prefer callbacks that cause application state to update later.

## 9.5 Units

Do not erase units into floats in the semantic layer.

These should remain distinct:

```text
12px
20%
250ms
30deg
```

Reject meaningless assignments at compile time.

---

# 10. Module/import/component system

## 10.1 Imports

Support familiar named imports:

```text
import { Button, Text } from "@argui/ui"
import { ProjectCard } from "./components/project-card.argui"
import { Project as ProjectModel } from "./models.argui"
```

Support:

- relative imports;
- configured include paths;
- package/library imports beginning with `@`;
- aliases;
- named re-exports;
- wildcard re-export only if semantics stay deterministic;
- import-cycle diagnostics.

## 10.2 Components

Example direction:

```text
export component ProjectCard {
    in property project: Project
    out property selected: bool
    callback open(project_id: string)

    Card {
        Text {
            text: project.name
        }
    }
}
```

Support:

```text
private property
in property
out property
in-out property
```

## 10.3 Slots

Reusable components need slots.

Support a clear model such as:

```text
export component Panel {
    slot header
    slot content

    Column {
        header
        content
    }
}
```

Unnamed default children may be sugar for a default slot.

Do not rely on arbitrary AST substitution.

## 10.4 Repetition

Example:

```text
for project in projects key project.id {
    ProjectCard {
        project: project
    }
}
```

Stateful repeaters should require or strongly warn for missing keys.

The runtime model needs incremental operations:

- insert;
- remove;
- move;
- update;
- reset.

Virtualized list integration must not force expansion of an entire large model.

## 10.5 Conditions

Support source-site-stable branches:

```text
if loading {
    Spinner {}
} else {
    Content {}
}
```

Each branch's nodes keep their own site identities.

---

# 11. Properties, bindings, expressions

## 11.1 One-way bindings

```text
Text {
    text: "Hello " + user.name
}
```

Compiler records all property dependencies.

## 11.2 Two-way bindings

Use a dedicated operator:

```text
Input {
    value <=> search
}
```

Require type compatibility.

## 11.3 Callback/event blocks

Example:

```text
Button {
    on click {
        count += 1
        save(count)
    }
}
```

The DSL event language should allow UI-local mutations and callback calls, but not filesystem/network/database APIs.

## 11.4 Pure functions

If functions are supported, distinguish pure expression helpers from callback/event side effects.

Pure functions may participate in bindings.
Impure functions may not be silently evaluated from reactive bindings.

## 11.5 Binding cycles

Detect static cycles when possible.

Runtime cycle detection remains necessary for dynamic dependencies.

Diagnostics should show the cycle path.

---

# 12. Theme, tokens, and style system

This is a first-class language subsystem, not a few color variables.

## 12.1 Theme tokens

Example:

```text
export theme AppTheme {
    --background: color = #0b0c10
    --surface: color = #13151b
    --foreground: color = #f5f7fa

    --accent: color = #735cff
    --accent-hover: color = #856fff

    --space-xs: length = 4px
    --space-sm: length = 8px
    --space-md: length = 16px
    --space-lg: length = 24px

    --radius-sm: length = 6px
    --radius-md: length = 10px

    --motion-fast: duration = 120ms
    --motion-normal: duration = 220ms
}
```

## 12.2 Token references

```text
padding: var(--space-md)
background: var(--surface)
```

The compiler resolves token references to typed token IDs. Runtime code should not repeatedly hash token strings.

## 12.3 Derived tokens

Allow tokens to depend on other tokens:

```text
--button-bg: color = var(--accent)
```

Detect cycles.

## 12.4 Theme modes

Support light/dark/system or user-defined theme variants without forcing the entire language to know platform theme details.

Example direction:

```text
theme AppTheme {
    light {
        --background: #ffffff
    }

    dark {
        --background: #0b0c10
    }
}
```

## 12.5 Styles

Named styles:

```text
export style PrimaryButton for Button {
    background: var(--accent)
    foreground: var(--on-accent)
    radius: var(--radius-md)
    padding: var(--space-md)

    hovered {
        background: var(--accent-hover)
    }
}
```

## 12.6 Deterministic precedence

Do not implement CSS specificity.

Use a documented order such as:

```text
native/component defaults
    ↓
active theme defaults
    ↓
component variant
    ↓
named styles
    ↓
component-local style
    ↓
inline properties
    ↓
active state overrides
    ↓
animation/transition resolved value
```

Exact ordering must be specified and tested.

## 12.7 Variants

Example:

```text
Button {
    variant: destructive
}
```

Variant appearance should come from the component/style/theme system, not hard-coded application colors.

---

# 13. States, transitions, and animation

## 13.1 States

Support named visual states driven by expressions and built-in interaction states.

Example direction:

```text
states {
    disabled when !enabled {
        opacity: 0.5
    }

    open when expanded {
        height: 300px
    }
}
```

## 13.2 Transitions

Allow entering/leaving/in-out transition policies.

## 13.3 Property animation

Example:

```text
animate width {
    spring {
        stiffness: 220
        damping: 24
    }
}
```

and:

```text
animate opacity {
    duration: var(--motion-fast)
    easing: ease-out
}
```

## 13.4 Keyframes

Support keyframes using the existing Argui timeline engine.

## 13.5 DSL animation store

`argui-dsl-runtime` owns:

```text
AnimationKey {
    component_instance
    source_site
    property
    animation_slot
}
```

Map that key to persistent `Motion<T>` handles.

On compatible reload:

- reuse the existing motion;
- update/retarget the driver/spec;
- preserve current value;
- preserve spring velocity when the underlying Argui API supports it.

On incompatible property type change:

- discard the old motion;
- initialize from the new definition.

## 13.6 Reduced motion

The DSL runtime/compiler must respect Argui's existing reduced-motion behavior.

No DSL animation may bypass accessibility reduced-motion policy unless an explicitly reviewed essential-motion escape hatch exists.

---

# 14. WGSL design

## 14.1 Never invent a WGSL DSL

Use standard WGSL files.

Example DSL declaration:

```text
export effect Glow {
    shader: "../shaders/glow.wgsl"

    parameter intensity: float = 0.5
    parameter tint: color = #7c5cff
}
```

Usage:

```text
Card {
    effect: Glow {
        intensity: hovered ? 1.0 : 0.35
        tint: var(--accent)
    }
}
```

## 14.2 Compile-time validation

Release build:

```text
.argui effect declaration
        +
.wgsl
        ↓
argui-shader/Naga validation
        ↓
typed effect metadata
        ↓
generated Rust registration
```

Fail the build on invalid WGSL.

## 14.3 Dev hot reload

```text
.wgsl save
   ↓
compiler service reads source
   ↓
Naga + Argui ABI validation
   ↓
prepare GPU pipeline
   ↓
atomic swap
```

On error:

- report source-mapped diagnostic;
- keep old effect definition/pipeline.

## 14.4 Multi-pass effects

Preserve Argui's multi-pass effect model and input declarations.

The DSL should describe pass composition/reference to actual WGSL files, not inline a second shading language.

## 14.5 WGSL parameters and animation

Effect parameters remain typed DSL properties and may be bound/animated through existing Argui property animation support where supported.

---

# 15. Assets

## 15.1 Static imports

Support paths such as:

```text
Image {
    source: asset("./assets/logo.png")
}
```

SVGs should route to vector support when appropriate.

## 15.2 Dependency tracking

Imported asset files become compiler dependencies so:

- Cargo rebuilds when assets change;
- `argui dev` hot-reloads changed assets.

## 15.3 Reachability

Release compiler includes only reachable assets from reachable components/effects.

## 15.4 Stable identity

Asset registry maintains stable identity across content edits and a revision/content hash for re-upload.

---

# 16. Localization

The DSL must integrate with the existing Fluent-based `argui-i18n` capability.

Do not create a competing localization system.

Suggested expression surface:

```text
Text {
    text: tr("editor.save")
}
```

with typed/interpolated arguments where supported.

The semantic compiler should validate known message references when catalogs are available.

Live development should allow locale/catalog changes without requiring a Rust rebuild where practical.

---

# 17. Accessibility

Accessibility is not an optional DSL afterthought.

Requirements:

- official `@argui/ui` components expose correct semantics by default;
- custom components can expose/forward accessible labels, descriptions, state, roles, and actions;
- LSP/doc schema documents accessibility-relevant properties;
- generated and live backends must produce equivalent semantics trees;
- hidden/inert states are modeled explicitly;
- theme/style changes must not accidentally discard semantics.

---

# 18. Standard library strategy

The final public UI surface should be:

```text
import { Button, Input, Card, Dialog, ... } from "@argui/ui"
```

not Rust widget builders.

## 18.1 Stage 1

Expose current Rust widgets/native behaviors through schema adapters so the DSL can reach feature parity quickly.

## 18.2 Stage 2

Move visual composition and styles of official controls into `.argui`.

Keep complex native behavior in small Rust primitives/controllers.

## 18.3 Stage 3

Make the `.argui` standard library the canonical public control library.

`argui-widgets` may remain an implementation crate, but docs/app authors should no longer need its Rust API.

## 18.4 Tree shaking

The release compiler determines reachable standard-library components and emits/includes only those needed by the application and their transitive dependencies.

---

# 19. Typed IR

IR means **Intermediate Representation**.

The typed IR is the canonical semantic bridge shared by live execution and release codegen.

It is not source syntax and not Rust syntax.

## 19.1 Requirements

Typed IR must contain normalized representations of:

- modules;
- imports resolved to definition IDs;
- components;
- component public ABI;
- native primitive references;
- properties;
- bindings;
- dependency edges;
- callbacks;
- events;
- expressions;
- repeaters;
- slots;
- conditions;
- styles;
- theme-token references;
- states;
- transitions;
- animations;
- effects;
- assets;
- accessibility metadata;
- optional development source spans.

## 19.2 IDs

Compiler-side IDs should be explicit:

```text
FileId
ModuleId
DefinitionId
ComponentId
PropertyId
CallbackId
ExpressionId
StyleId
ThemeId
ThemeTokenId
EffectDefinitionId
AssetId
SiteId
```

These are compiler/IR identities and must not be confused with runtime engine `Name`.

## 19.3 Stable site identity

`SiteId` must remain stable across whitespace/comments and unrelated source edits.

Do not derive it directly from line numbers.

Recommended inputs:

- containing component semantic identity;
- explicit source-node identity from the syntax tree/definition map;
- optional compiler-maintained persistent source fingerprint.

A source edit that genuinely deletes/recreates a site may legitimately change identity.

---

# 20. Live runtime design

`argui-dsl-runtime` is development-only application code.

It should be type-erased where needed, but bounded and explicit.

## 20.1 Runtime values

Example:

```text
DslValue
  Bool
  Int
  Float
  String
  Length
  Duration
  Color
  Brush
  Struct
  Enum
  Model handle
  Asset handle
  ...
```

Keep this out of core engine crates.

## 20.2 Component instance state

Each live instance owns:

```text
ComponentInstance
├── instance id
├── definition id
├── dynamic properties
├── child instances
├── callback bindings
├── repeater state
├── animation store references
├── theme dependencies
└── retained identity map
```

## 20.3 Expression execution

Use a compact typed IR evaluator/bytecode for development.

Do not evaluate source AST directly on every frame.

Requirements:

- pre-resolved property accesses;
- pre-resolved callback references;
- typed opcodes/operations where useful;
- no string name lookups in hot expression paths;
- no arbitrary Rust reflection.

---

# 21. Live reload protocol and process model

For the best cross-platform design, do not require the DSL compiler to be linked into the running application.

## 21.1 Development host

`argui dev` runs a host-side compiler/watch service:

```text
filesystem watcher
       ↓
incremental compiler
       ↓
validated typed live package
       ↓
versioned protocol
       ↓
running app's dsl-runtime
```

Benefits:

- browser/mobile can receive updates from a desktop host;
- compiler memory is outside the application;
- compiler crashes/errors do not kill the app;
- application only links `argui-dsl-runtime` in dev;
- same architecture works for desktop/web/mobile.

## 21.2 Transport abstraction

Support:

- desktop local socket/TCP;
- WebSocket for browser;
- host-to-device WebSocket/TCP for Android/iOS.

The protocol must be transport-independent.

## 21.3 Versioned protocol

Every live package starts with:

```text
protocol version
IR format version
Argui engine compatibility version
public root API hash
generation
```

Reject incompatible compiler/runtime versions with a clear diagnostic.

Do not attempt best-effort decoding of unknown schema versions.

---

# 22. Transactional hot reload

Reload must be all-or-nothing.

```text
changed files
      ↓
incremental parse
      ↓
module resolution
      ↓
type checking
      ↓
binding validation
      ↓
WGSL validation
      ↓
asset validation
      ↓
typed IR package
      ↓
compatibility analysis
      ↓
prepare state migration
      ↓
prepare shader/resources
      ↓
atomic commit
      ↓
rebuild affected component roots
      ↓
UiTree reconciliation
```

If any pre-commit phase fails:

```text
diagnostics emitted
previous generation stays active
```

No partial UI generation may leak into the mounted application.

---

# 23. Hot reload state model

Treat three state layers separately.

## 23.1 Business/application state

Owned by user Rust:

```text
documents
projects
network state
database state
user session
```

Must remain alive because the process remains alive.

## 23.2 Declarative component state

Owned by DSL runtime:

```text
expanded
selected_tab
draft text property
local count
animation progress
```

Preserve on reload when identity/type are compatible.

## 23.3 Retained UI state

Owned by Argui `UiTree`:

```text
focus
selection
text editor internals
scroll offsets
hover/pressed/focus-visible
gesture state
caret state
```

Preserve through stable retained identity and existing reconciliation.

---

# 24. Declarative state migration rules

For each component instance:

Preserve a property value when:

```text
same component instance identity
AND same semantic PropertyId/name
AND old/new types are migration-compatible
```

New property:

```text
initialize new default/binding
```

Removed property:

```text
drop state
```

Type changed incompatibly:

```text
drop old value
initialize new value
emit development note if useful
```

For structs, field-wise migration may be supported only if the semantic rules remain deterministic.

Do not perform surprising implicit string/number conversions just to preserve state.

---

# 25. Public component ABI and restart boundary

A root component exposes an ABI to Rust.

Example:

```text
export component Editor {
    in property document: Document
    out property dirty: bool
    callback save(path: string)
}
```

Compiler produces a deterministic `PublicApiHash`.

During live reload:

- internal property/style/layout changes are reloadable;
- changing a private/internal type is reloadable if migration rules allow;
- changing the public ABI used by Rust should report **restart required**.

Do not crash or silently bind the old Rust callback to a new incompatible signature.

This mirrors the useful boundary used by Slint live preview: UI internals can reload, while native-language interface changes require rebuilding.

---

# 26. Release AOT integration

Follow a build-time model similar in spirit to Slint's `slint-build`.

## 26.1 Application Cargo setup

Target user experience:

```toml
[dependencies]
argui = "..."

[build-dependencies]
argui-dsl-build = "..."
```

Generated by `argui init`; users should not hand-author boilerplate repeatedly.

`build.rs`:

```rust
fn main() {
    argui_dsl_build::compile("ui/main.argui").unwrap();
}
```

Application code:

```rust
argui::include_ui!();
```

Exact API naming may differ, but keep it minimal.

## 26.2 Generated Rust

Generate:

- typed public component bindings;
- typed property storage;
- bindings using `argui-reactive`;
- component render/build functions;
- stable source-site retained identities;
- event bridges;
- typed theme token references;
- effect definitions;
- reachable asset registrations.

Do not generate a generic runtime AST evaluator for release.

## 26.3 Reachability/DCE

Before Rust emission, compute transitive reachability from exported roots.

Drop:

- unused components;
- unused styles;
- unused theme definitions when safe;
- unused effects;
- unused shaders;
- unused assets;
- unused helper expressions/functions.

## 26.4 Generated-code API stability

Generated code may use a dedicated internal/publicly-hidden Argui codegen support API.

Do not require generated output to mimic hand-written builder chains if a cleaner lower-level construction API is faster and more stable.

Compiler and engine versions must be version-matched.

---

# 27. Development integration

## 27.1 `argui dev`

`argui dev` should:

1. start compiler/watch service;
2. invoke Cargo with the dev-only DSL runtime feature;
3. launch/attach to the target;
4. establish live protocol;
5. send initial compiled module;
6. stream subsequent accepted generations;
7. surface diagnostics.

## 27.2 No CLI requirement for release

Ordinary `cargo build --release` remains valid through `build.rs`.

The CLI is a developer-experience tool, not a runtime requirement.

## 27.3 Web

For WebAssembly:

- compiler runs on host;
- browser app contains dev-only `dsl-runtime`;
- updates arrive over WebSocket;
- no filesystem watcher/compiler is compiled into WASM.

## 27.4 Android/iOS

Same host compiler and transport concept.

Do not attempt to compile the DSL on-device for normal development.

---

# 28. Rust business-logic integration

Users should never need to implement UI in Rust.

Generated component bindings provide typed Rust integration.

Possible shape:

```rust
let ui = MainWindow::new(app_state)?;

ui.on_create_project(|state| {
    state.create_project();
});

ui.set_user(user);
```

or generated traits if that integrates better with Argui's current model runtime.

Requirements:

- generated API contains no `DslValue`;
- compile-time Rust types for public UI API;
- no string-based callback/property access in release application code;
- user business model remains ordinary Rust;
- UI callbacks can enqueue commands/tasks through documented application APIs.

---

# 29. Subsecond / current `hot-reload` feature retirement

## 29.1 Current reality

There is currently **not a separate Argui hot-reload crate**.

The current system is:

```text
argui feature "hot-reload"
  -> argui-runtime feature "hot-reload"
  -> dioxus-devtools/Subsecond
  -> crates/argui-runtime/src/hot_reload.rs
```

It patches Rust code in native debug builds.

## 29.2 Why it should stop being Argui's primary UI hot reload

Once DSL live reload is complete, Subsecond is inferior for the primary UI authoring path because it:

- recompiles/patches Rust instead of reloading declarative UI;
- is constrained by Rust type layout/signature changes;
- requires tip-crate executable organization;
- is native-debug-oriented;
- does not provide the intended WebAssembly/mobile live architecture;
- creates a second, conceptually competing UI hot-reload system;
- adds `dioxus-devtools` dependency/build complexity.

The DSL live runtime can preserve UI state based on semantic component/site identity rather than Rust function patching.

## 29.3 Retirement policy

The repository's code-quality policy says replaced APIs should be removed rather than maintained as long-lived shims.

Therefore do **not** create a permanent deprecated alias.

Use staged project migration:

### Transition milestone

Keep current Subsecond support until DSL live reload reaches required parity.

Update docs to call it legacy Rust UI hot patching during this short transition only.

### Removal milestone

In one coordinated change remove:

- `argui` `hot-reload` feature;
- `argui-runtime` `hot-reload` feature;
- `dioxus-devtools` dependency from workspace/runtime if unused elsewhere;
- `crates/argui-runtime/src/hot_reload.rs`;
- current hot-reload gallery page;
- `docs/hot-reload.md` content describing the old primary path;
- README claims/instructions for Subsecond UI authoring;
- any `RuntimeEvent::HotReloaded` API that only exists for Subsecond.

Replace documentation with:

```text
argui dev
```

and DSL live reload.

## 29.4 Rust business-logic hot patching

Do not make Subsecond a core dependency merely to patch application logic.

Initial policy after removal:

- `.argui` UI changes: instant DSL hot reload;
- `.wgsl` changes: instant shader hot reload;
- Rust business-logic changes: normal rebuild/restart.

If demand exists later, a **separate optional companion integration** may provide Rust hot patching. It must not be required by the DSL architecture or core runtime.

---

# 30. DevTools integration

Existing Argui DevTools should understand DSL source metadata.

Add development views for:

- component name;
- source location;
- source site ID;
- resolved styles;
- active states;
- theme tokens read by selected node;
- property values/binding sources;
- active animations;
- effect/shader name and revision;
- reload generation.

Potential future editing:

- changing theme token values live;
- jumping from inspected node to source;
- toggling component state for preview.

Do not make DevTools necessary for DSL runtime correctness.

---

# 31. LSP architecture

The LSP must be thin.

Never duplicate semantic-language logic in the LSP.

It calls `argui-dsl-semantic` queries.

## 31.1 Minimum capabilities

- parse/type diagnostics;
- import diagnostics;
- unknown component/property diagnostics;
- completion;
- hover;
- go to definition;
- find references;
- rename;
- document/workspace symbols;
- semantic highlighting;
- formatting;
- color presentation;
- code actions for common fixes.

## 31.2 Context-sensitive theme completion

For:

```text
padding: var(--
```

the semantic engine knows the expected type.

Only compatible theme tokens should be proposed.

## 31.3 Component property completion

For:

```text
Button {
    |
}
```

completion comes from the canonical component/native schema.

## 31.4 Shared AI interface

CLI should expose the same semantic data in machine-readable form:

```text
argui check --json
argui schema Button --json
argui complete ui/main.argui 42:18 --json
argui symbols --json
```

This makes the DSL especially agent-friendly.

---

# 32. Formatter

Because the language is intended to be generated/edited by AI as well as humans, formatter behavior must be deterministic.

Requirements:

- lossless comments;
- stable output;
- no semantic changes;
- clear multiline rules;
- trailing separators policy;
- import sorting/grouping;
- idempotence test.

Formatting must operate from CST, not from typed IR.

---

# 33. Testing strategy

Tests are not optional; dev/release semantic divergence is the biggest architectural risk.

## 33.1 Core pre-DSL tests

Add tests for:

- dynamic/static `Name` equality/hash;
- retained identity preservation under source-like reorders;
- typed theme token storage;
- selective theme invalidation;
- effect registry dynamic replacement;
- invalid replacement keeps previous effect;
- shader source-map diagnostics;
- asset stable identity/revision.

## 33.2 Parser tests

Golden fixtures for:

- all declarations;
- malformed constructs;
- recovery;
- comments/whitespace;
- incomplete editor states.

## 33.3 Semantic tests

- imports;
- aliases;
- re-exports;
- module cycles;
- type inference;
- type mismatches;
- property visibility;
- callback signatures;
- two-way binding compatibility;
- binding cycles;
- style target validation;
- theme token type checking;
- state/transition property checking;
- effect parameter checking;
- asset paths.

## 33.4 Live migration tests

For each reload scenario assert:

- old property preservation;
- new property defaults;
- removed property cleanup;
- incompatible type reset;
- repeater key preservation;
- focus preservation;
- text input preservation;
- scroll preservation;
- animation retarget;
- callback identity preservation;
- public API change returns restart-required.

## 33.5 WGSL hot reload tests

- valid source accepted;
- syntax error rejected;
- validation error rejected;
- old pipeline retained after failure;
- fixed source accepted after failure;
- parameter schema expansion resizes storage;
- line/column mapping points to user file.

## 33.6 AOT/live parity suite

This is critical.

For fixture `.argui` applications:

1. execute through live runtime;
2. execute generated Rust version;
3. feed the same event/state changes;
4. compare normalized:
   - Element tree;
   - semantics;
   - layout-affecting properties;
   - paint properties;
   - callback outputs;
   - property values.

The parity suite should catch compiler/runtime semantic drift.

## 33.7 Release dependency gate

Create a minimal release fixture and assert normal dependencies do not include:

```text
argui-dsl-parser
argui-dsl-semantic
argui-dsl-compiler
argui-dsl-runtime
argui-dsl-lsp
file-watching crates
dev protocol/server
```

The generated application may depend on `argui-reactive` and normal engine crates because they are part of production semantics, not a DSL interpreter.

---

# 34. Performance requirements

## 34.1 Idle

An idle live or generated UI must not:

- continuously poll files inside the app;
- request frames without active animation;
- reevaluate clean bindings;
- rebuild unchanged subtrees.

The external `argui dev` service owns file watching.

## 34.2 Incremental compile

A change to one leaf component should not reparse/type-check every project file.

## 34.3 Runtime

Pre-resolve:

- property IDs;
- component references;
- callbacks;
- native property adapters;
- theme token IDs;
- effect IDs.

Avoid hot-path string lookups.

## 34.4 Release

Generated code should use typed values and direct calls, not `HashMap<String, DslValue>`.

---

# 35. Implementation phases

The implementation agent must complete these in order.

---

## Phase A — Architecture contracts before coding the language

Create ADR/design docs for:

1. engine `Name` type;
2. retained identity;
3. reactive property core;
4. theme v2;
5. native declarative schema;
6. dynamic effect registry;
7. WGSL validation extraction;
8. asset identity/revision;
9. live protocol compatibility;
10. public UI ABI/restart boundary.

Do not freeze `.argui` grammar before these contracts are reviewed.

### Exit criteria

All relevant engine owners/types and dependency directions are known.

---

## Phase B — Dynamic engine names

Implement engine `Name` and migrate static-only identifiers.

### Exit criteria

No DSL-relevant engine identity requires `'static` source text.

---

## Phase C — Retained source identity

Add private compiler/runtime retained identity to `Element` reconciliation.

### Exit criteria

Reorder/insertion tests preserve state as specified.

---

## Phase D — `argui-reactive`

Implement typed reactive properties/transactions/dependency tracking.

### Exit criteria

Typed property binding tests pass and no DSL types exist in the crate.

---

## Phase E — Theme v2

Refactor `argui-theme` to typed tokens and dependency-friendly runtime switching.

Do not yet design visual DSL syntax beyond semantic requirements.

### Exit criteria

A Rust-only test application can define typed tokens, switch values at runtime, and precisely invalidate consumers.

---

## Phase F — Declarative native schema

Create `argui-schema`.

Expose a minimal native primitive surface and adapters.

### Exit criteria

A generic non-parser test can construct a native element by schema IDs/typed values without hard-coding widget names in a compiler.

---

## Phase G — Shader/effect live readiness

Create `argui-shader`, move validation/ABI logic, make effect definitions dynamic/revisioned, implement replacement and source mapping.

### Exit criteria

A Rust-only test can hot-swap WGSL safely without DSL.

---

## Phase H — Asset live readiness

Introduce stable source asset registry/revisions.

### Exit criteria

Changing image/SVG bytes updates a stable logical asset.

---

## Phase I — DSL syntax crate + parser

Now implement language syntax.

Do not combine lexer, parser, semantic analysis, and codegen in one crate/file.

### Exit criteria

Lossless CST + AST facade + diagnostics/recovery.

---

## Phase J — Modules and semantic compiler

Implement:

- file/module graph;
- imports;
- exports;
- aliases;
- structs/enums;
- component declarations;
- properties;
- callbacks;
- slots;
- type system;
- expressions;
- repeaters;
- themes/styles;
- states/transitions/animations;
- effects/assets.

### Exit criteria

`argui check` can fully validate multi-file projects without generating Rust.

---

## Phase K — Typed IR

Lower semantic representation to stable typed IR.

### Exit criteria

IR contains no unresolved strings for semantic references where IDs should be used.

---

## Phase L — Release Rust backend first

Implement `dsl-compiler` Rust AOT codegen and `dsl-build`.

This gives a production path before live runtime complexity.

### Exit criteria

A multi-file `.argui` application builds and runs natively with no DSL compiler/runtime in normal dependencies.

---

## Phase M — Live runtime

Implement dynamic typed component runtime and expression evaluator.

### Exit criteria

The same fixture UI works live and passes AOT/live parity tests.

---

## Phase N — Dev compiler service/protocol

Implement `dsl-protocol` and `argui dev`.

### Exit criteria

Native desktop `.argui` and `.wgsl` edits reload transactionally without Rust recompilation.

---

## Phase O — Standard library migration

Make `@argui/ui` the primary public component library.

Initially wrap/adapt existing widgets; progressively move appearance/composition into DSL.

### Exit criteria

Main documentation/sample apps no longer author UI through Rust widget builders.

---

## Phase P — Web/mobile live transports

### Exit criteria

- browser receives live packages from host compiler;
- Android/iOS development apps can connect to host compiler;
- release builds remain unaffected.

---

## Phase Q — LSP + formatter + AI CLI

### Exit criteria

Production-quality authoring loop.

---

## Phase R — Retire Subsecond Argui UI hot reload

Only after DSL live reload parity is reached.

### Exit criteria

Current `hot-reload` features/dependency/docs/bridge removed in one coordinated change.

---

# 36. Proposed language surface (direction, not frozen grammar)

This exists to ensure the architecture covers required concepts. Do not treat punctuation as final until semantic contracts are stable.

```text
import {
    Button,
    Column,
    Text,
    Input
} from "@argui/ui"

import { ProjectCard } from "./components/project-card.argui"
import { AppTheme, Heading1 } from "./theme.argui"

export struct Project {
    id: string
    name: string
}

export component Dashboard {
    in property title: string
    in-out property search: string = ""
    in property projects: model<Project>

    callback create_project()
    callback open_project(id: string)

    Column #root {
        gap: var(--space-lg)
        padding: var(--space-lg)

        Text {
            text: title
            style: Heading1
        }

        Input #search_input {
            value <=> search
            placeholder: tr("dashboard.search")
        }

        for project in projects key project.id {
            ProjectCard {
                project: project

                on open {
                    open_project(project.id)
                }
            }
        }

        Button {
            text: tr("dashboard.new_project")
            variant: primary

            on click {
                create_project()
            }
        }
    }
}
```

Theme:

```text
export theme AppTheme {
    --background: color = #0b0c10
    --surface: color = #13151b
    --foreground: color = #f7f7fa
    --accent: color = #735cff

    --space-sm: length = 8px
    --space-md: length = 16px
    --space-lg: length = 24px

    --radius-md: length = 10px
    --motion-fast: duration = 120ms
}

export style Heading1 for Text {
    font-size: 28px
    font-weight: 700
    foreground: var(--foreground)
}
```

Animation/state:

```text
states {
    compact when width < 600px {
        sidebar-width: 0px
    }
}

animate sidebar-width {
    spring {
        stiffness: 220
        damping: 24
    }
}
```

WGSL effect:

```text
export effect Glow {
    shader: "../shaders/glow.wgsl"

    parameter intensity: float = 0.35
    parameter tint: color = var(--accent)
}
```

---

# 37. Documentation/product migration

When the DSL is production-ready:

## README

Primary example becomes `.argui`, not a Rust `Render` implementation.

## Getting started

Flow:

```text
cargo install argui-cli
argui new my-app
cd my-app
argui dev
```

## Rust docs

Describe Rust UI builders as:

```text
engine-level / advanced native extension API
```

not the normal authoring path.

## Component docs

Documentation should be generated from the same component/schema metadata used by the LSP.

---

# 38. AI-agent-specific design requirements

The DSL is intended to be easy for agents to author.

Therefore:

- deterministic formatter;
- strict type system;
- no implicit CSS cascade;
- no hidden global mutation;
- concise diagnostics;
- machine-readable `argui check --json`;
- schema query command;
- exact available-property completion;
- source-level stable component names;
- no need for an agent to inspect Rust implementation to discover a widget API.

An AI should be able to perform this loop:

```text
argui schema Dialog --json
        ↓
edit ui/settings.argui
        ↓
argui check --json
        ↓
fix diagnostics
        ↓
preview/screenshot/inspect
```

---

# 39. Important anti-patterns to reject in review

Reject implementations that:

- leak source strings to obtain `'static` references;
- make core engine crates depend on `argui-dsl-*`;
- use one 2,000-line parser/compiler file;
- put tests under `src/`;
- implement dev runtime and release semantics separately without parity tests;
- recompile Rust for normal `.argui` live reload;
- ship parser/compiler/runtime interpreter in release by default;
- hard-code every widget name in a giant compiler match;
- use string property lookup in release hot paths;
- create CSS specificity/cascade rules;
- embed arbitrary Rust in `.argui`;
- create a custom shader language instead of WGSL;
- discard the working UI on a failed reload;
- silently reuse component state across incompatible type changes;
- restart every animation on any source save;
- identify source sites by line number;
- treat repeater index as stable identity when a user key exists;
- require the LSP to reimplement type/import resolution;
- keep Subsecond as a second primary UI hot reload path after DSL live reload ships.

---

# 40. Definition of done for “DSL is the primary Argui UI API”

The project is not done merely because `.argui` can render a Button.

The DSL becomes the primary path only when all of the following are true:

- multi-file imports/components work;
- components have typed properties/callbacks/slots;
- structs/enums/models work;
- one-way/two-way bindings work;
- themes/tokens/styles/variants work;
- runtime theme switching works;
- states/transitions/animations work;
- existing Argui animation capabilities are reachable where semantically valid;
- WGSL effects work from real `.wgsl` files;
- WGSL hot reload is transactional;
- assets hot reload;
- accessibility remains correct;
- localization integrates;
- focus/text/scroll state survives compatible reload;
- component state migration works;
- public ABI changes clearly request restart;
- AOT/live parity suite passes;
- release builds contain no DSL compiler/interpreter/live tooling;
- `argui dev` works without recompiling Rust for UI/WGSL edits;
- LSP supports diagnostics/completion/navigation/rename/formatting;
- official docs/examples use DSL first;
- official `@argui/ui` is usable without looking at Rust;
- old Subsecond UI hot reload path is retired.

---

# 41. Slint design lessons intentionally adopted

Argui should learn from, not clone blindly, Slint's proven separation:

1. `.slint` files are normally AOT compiled for Rust/C++ applications.
2. Slint live preview uses an interpreter at runtime for development.
3. Live preview keeps business logic connected and preserves properties/callbacks/models.
4. Invalid edited UI does not replace the previous functioning preview.
5. Native-language public API changes remain a rebuild/restart boundary.
6. Slint has a build-time Rust helper (`slint-build`) that compiles UI from `build.rs`.
7. Slint's language has typed properties/bindings, two-way bindings, imports, components, states, and transitions.

Argui should retain its own strengths:

- retained `UiTree`;
- update-phase classification;
- WGPU renderer;
- existing animation engine;
- custom WGSL effects;
- Rust-native business logic;
- strong accessibility;
- explicit theme system;
- no browser/DOM model.

Reference documentation consulted while writing this plan:

- https://docs.slint.dev/latest/docs/slint/guide/tooling/live-preview/
- https://docs.slint.dev/latest/docs/rust/slint_build/
- https://docs.slint.dev/latest/docs/slint/reference/language/imports/
- https://docs.slint.dev/latest/docs/slint/reference/language/properties/
- https://docs.slint.dev/latest/docs/slint/reference/language/bindings/
- https://docs.slint.dev/latest/docs/slint/reference/language/two-way-bindings/
- https://docs.slint.dev/latest/docs/slint/reference/language/states-and-transitions/

---

# 42. First instruction to the implementation agent

**Do not start by writing `main.argui` parser code.**

Start with Phases A through H.

The immediate first PR/changeset should:

1. write the architecture contracts/ADRs;
2. implement the engine `Name` abstraction and migrate the DSL-blocking `'static` identifiers;
3. add retained compiler/runtime element identity support with preservation tests;
4. establish the design/skeleton for `argui-reactive`;
5. design Theme v2 and the lightweight declarative schema boundary;
6. extract shader validation/ABI to a renderer-independent crate;
7. make effects/assets revision-ready.

Only after those engine boundaries are proven should the project freeze and implement the actual `.argui` grammar.

The reason is simple:

> The grammar is cheap to change before users exist. The engine contracts are expensive to change after the compiler, live runtime, code generator, LSP, standard library, and user applications depend on them.

Build the semantic foundation first.
