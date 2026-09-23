# Simplify the public API and add application testing

Status: implemented locally; protected-branch verification pending

Target release: `0.3.0`

This document is an implementation plan, not a collection of optional ideas. Its
goal is to make ordinary Argui applications easy to write and test without
removing the low-level event system that advanced applications need.

An agent implementing this plan must continue through every phase and satisfy
the complete definition of done. Finishing the first vertical slice, making the
examples compile, or leaving follow-up TODOs is not completion. If external
infrastructure is unavailable, finish every unblocked item, record the exact
blocker and the command or external action needed, and do not describe the plan
as complete.

## Required reading

Before changing code, read these documents completely:

- [`contributing/code-quality.md`](contributing/code-quality.md)
- [`architecture.md`](architecture.md)
- [`ui/interaction.md`](ui/interaction.md)
- [`runtime/models.md`](runtime/models.md)
- [`contributing/releases.md`](contributing/releases.md)

For graphical Linux validation, also read
[`contributing/linux-testing.md`](contributing/linux-testing.md) and keep every
window on the hidden display.

## Problem statement

The low-level event system is capable, but it is currently also the main path
shown to new users. A basic button often requires all of the following:

- attaching a listener to a parent container;
- selecting `EventType::Click` manually;
- comparing `event.target_key()`;
- matching `UiEventKind::Click`;
- remembering to call `cx.notify()`.

This makes a simple counter look like an event-delegation example. The same
pattern creates large application-wide event handlers in the Widget Gallery,
even though most controls have a single local reaction.

Argui also has the internal pieces required for headless tests, but no public
application harness combines them. Users should not have to recreate
`SingleWindowModel`, `UiTree`, layout, event delivery and reconciliation merely
to click a button in a unit test.

## Product principles

The implementation must follow these principles:

1. The common path is local, typed and short.
2. The advanced path remains explicit and powerful.
3. Bubbling, capture, passive listeners, one-shot listeners,
   `prevent_default()` and propagation control are not removed.
4. Direct widget callbacks must use the same underlying event pipeline as
   low-level listeners. There must not be a second dispatch system.
5. Closures remain in the runtime handler registry. Do not store closures in
   `Element`; elements must remain cloneable, comparable and suitable for
   retained reconciliation and host transaction replacement.
6. Application state remains controlled by the application. Argui may retain
   temporary interaction state such as focus, pointer capture, drag state and
   typeahead state, but must not silently become the owner of business data.
7. A headless test and a native rendering test solve different problems. The
   public harness does not replace renderer and OS integration tests.
8. No compatibility shim, deprecated duplicate API, placeholder, `TODO`,
   `FIXME`, `todo!()` or `unimplemented!()` may be left behind.
9. Every new public function and method needs accurate Rustdoc, including
   parameters, return values, errors and panics where applicable.
10. All Rust source files must remain at or below 600 physical lines, and tests
    must live under `tests/` in the appropriate crate.

## Target user experience

### Simple local callback

The basic counter should require no event type, key comparison, event-kind
match or explicit invalidation:

```rust,ignore
impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Button::new("increment", "Increment", theme.button())
            .on_click(cx.callback(|counter| {
                counter.count += 1;
            }))
            .build()
    }
}
```

`Context::callback` is the convenience path for a local mutating callback. It
must invalidate the current presentation after the callback returns.

### Callback with the event and context

The direct widget path must also allow commands, focus changes and event
control:

```rust,ignore
Button::new("save", "Save", theme.button())
    .on_click(cx.event_handler(|editor, event, cx| {
        editor.save();
        event.stop_propagation();
        cx.notify();
    }))
    .build()
```

`Context::event_handler` does not hide the raw event and does not implicitly
invalidate. Its behavior must be documented clearly so users do not receive
surprising redraws.

### Advanced event delegation

The existing low-level API remains the documented advanced path:

```rust,ignore
Element::column(children).on(
    cx.listener(EventType::Click, |app, event, cx| {
        match event.target_key() {
            Some("save") => app.save(cx),
            Some("delete") => app.delete(cx),
            _ => {}
        }
    })
)
```

This is intentional event delegation, not the recommended implementation of a
single button.

### Headless application test

The minimum public harness should read like this:

```rust,ignore
use argui_testing::TestApp;

#[test]
fn increments_the_counter() {
    let mut app = TestApp::new(Counter::default());

    app.click("increment").unwrap();

    app.assert_text("Count: 1");
    app.assert_focused("increment");
}
```

Accessible queries must also be supported:

```rust,ignore
app.get_by_role(Role::Button, "Save").click().unwrap();
app.get_by_role(Role::TextInput, "Email")
    .type_text("person@example.com")
    .unwrap();
```

## Architectural decision

### Opaque handler identity

Add an opaque handler identity to `argui-ui`. The exact private representation
may reuse `EventHandlerId`, but application code must not construct or inspect
the identity.

The intended public shape is:

```rust,ignore
pub struct EventHandler { /* private */ }
```

It must be cheap to clone or copy and comparable wherever existing element
reconciliation requires comparison. It represents a callback registered by the
runtime but does not choose an event type.

An `EventHandler` must be convertible internally into an `EventListener` for a
specific `EventType`. Keep the construction surface as narrow as possible. Do
not expose handler owner IDs or slots as normal public application concepts.

### Runtime registration

Refactor handler registration in `argui-runtime` so that:

- `Context::callback` registers a simple state callback and automatically calls
  `cx.notify()` after successful delivery;
- `Context::event_handler` registers the existing full callback shape;
- `Context::listener(event_type, callback)` is implemented through the same
  registration primitive and retains its existing semantics;
- handler slot identity remains stable between equivalent renders;
- removed slots are never accidentally reused;
- multiple mounts of the same entity retain independent presentation handlers;
- routed child entities continue to receive their own handlers.

Do not make `argui-widgets` depend on concrete model types. Widgets receive an
opaque handler and attach an ordinary `EventListener` during `build()`.

### Direct listener semantics

A direct widget handler must fire only for that widget's own activation. It
must not fire because an unrelated interactive descendant bubbled a click
through the widget. Use an explicit target/current-target policy or an
equivalent dispatch filter if required; do not rely on undocumented tree shape.

The following behavior must remain consistent:

- mouse, touch, keyboard and accessibility activation converge on the same
  direct callback;
- disabled and busy controls do not invoke the callback;
- callback ordering is deterministic when more than one handler is attached;
- direct callbacks still participate in normal propagation;
- `stop_propagation`, `stop_immediate_propagation` and `prevent_default` retain
  their documented behavior;
- handler replacement during a rerender cannot call a stale closure.

## Phase 0: establish a green baseline

Do not begin the API migration while the protected branch or release checks are
red.

1. Inspect the latest protected-branch workflow rather than assuming the last
   local run is sufficient.
2. Fix the generated website catalogue mismatch and ensure every catalogue slug
   is unique.
3. Fix crates.io archive verification. All packages in the staged workspace
   must resolve the staged Argui versions instead of an older published crate.
4. Fix or split the quality workflow so it completes within its timeout without
   weakening the checks.
5. Run the relevant local targeted checks and confirm the protected workflow is
   green.
6. Record the baseline test count and coverage report for comparison after the
   migration.

At the time this plan was written, the failures included the duplicated
`text-selection` catalogue entry, archive verification resolving an older
`argui-ui 0.2.1`, and the quality job reaching its timeout. Recheck the current
state; do not blindly reproduce stale fixes.

## Phase 1: handler foundation

### `argui-ui`

1. Add the opaque `EventHandler` type beside `EventListener`.
2. Add the smallest internal/publicly-hidden conversion needed to bind a
   handler to an `EventType`.
3. If a direct-target filter is needed, model it explicitly in listener options
   and cover capture, target and bubble phases.
4. Re-export the new type from `argui-ui` and the top-level facade where
   appropriate.

### `argui-runtime`

1. Extract event-independent handler registration from `Context::listener`.
2. Implement `Context::callback` with automatic presentation invalidation.
3. Implement `Context::event_handler` with access to `UiEvent` and `Context` and
   no implicit invalidation.
4. Reimplement `Context::listener` using the shared registration path.
5. Preserve presentation-owned handler identity across host transaction replacement.

### Required tests

Add tests under `crates/argui-ui/tests/` and
`crates/argui-runtime/tests/model/` for:

- stable handler identity;
- callback replacement after rerender;
- removed handlers;
- nested and routed entities;
- separate mounts of one entity;
- automatic invalidation for `callback`;
- no automatic invalidation for `event_handler`;
- capture, target and bubble delivery;
- propagation cancellation;
- once and passive options;
- no delivery to hidden or closed presentations.

Run, with the final feature set retained between commands:

```sh
cargo nextest run -p argui-ui --all-features
cargo clippy -p argui-ui --all-targets --all-features -- -D warnings
cargo nextest run -p argui-runtime --all-features
cargo clippy -p argui-runtime --all-targets --all-features -- -D warnings
```

## Phase 2: button vertical slice

Implement the complete feature through one control before generalizing it.

### Public API

Add `Button::on_click(EventHandler) -> Button`. Support more than one handler
without silently replacing an earlier handler. During `build()`, attach the
listener to the actual interactive root produced by `ButtonBehavior`.

Do not remove `ButtonBehavior::action`; it is the runtime-neutral, controlled
behavior API and remains useful for event delegation and custom controls.

### Migration

1. Rewrite the Counter documentation example to use the direct button handler.
2. Keep the Events example as the canonical bubbling/delegation example.
3. Add a direct-handler example to the Button catalogue page.
4. Do not migrate unrelated widgets during this vertical slice.

### Required tests

Cover:

- pointer click;
- touch/tap activation;
- Enter and Space activation;
- accessibility click;
- disabled button;
- loading/busy button;
- two buttons with different callbacks;
- multiple callbacks on one button;
- direct callback followed by parent bubbling;
- propagation stopped by the direct callback;
- custom non-interactive button content;
- stale handler replacement after state changes.

The vertical slice is accepted only when the new Counter runs, its headless test
passes, the advanced event tests remain unchanged in behavior, and the relevant
WebAssembly check compiles.

## Phase 3: consistent widget event APIs

Inventory every interactive widget exported by `argui-widgets`. For each one,
either provide a direct handler API or document why the widget is intentionally
presentational. Do not leave controls with undocumented event handling.

Use this naming convention unless the existing typed action has a materially
better domain term:

| Control category | Direct API |
| --- | --- |
| Button-like activation | `on_click` or `on_activate` |
| Text editing | `on_input`, `on_submit` |
| Checkbox, switch, toggle | `on_change` |
| Radio, tabs, select | `on_select` |
| Range and slider | `on_change`, `on_commit` |
| Dialog, sheet, drawer, popover | `on_open_change`, `on_dismiss` |
| List, tree and table | `on_select`, `on_activate` |
| Menu and command controls | `on_action`, `on_open_change` |

Choose one name per semantic operation and apply it consistently. Do not add
aliases such as both `on_press` and `on_click` merely to accommodate taste.

### Migration order

Implement and verify controls in this order:

1. `Input`, `TextArea`, `InputOtp`.
2. `Checkbox`, `Switch`, `Toggle`, `ToggleGroup`.
3. `RadioGroup`, `Tabs`, `Select`, `NativeSelect`, `Combobox`.
4. `Range`, `Slider`, `ColorPicker`.
5. `Collapsible`, `Accordion`, `Dialog`, `AlertDialog`, `Sheet`, `Drawer`,
   `Popover`, `HoverCard`.
6. `Menu`, `ContextMenu`, `Menubar`, `CommandPalette`.
7. `Calendar`, `DatePicker`, `Pagination`, `Carousel`, `Breadcrumb`.
8. `List`, `VList`, `TreeView`, `Table`, `DataTable`.
9. Interactive composite widgets such as `Questionnaire`, `MessageScroller`,
   `Toast`, updater UI and sidebar controls.

### Payload policy

Direct handlers should receive the useful domain value when the event already
contains or deterministically produces one. Users should not repeat event-kind
matching for common cases.

Examples include:

- the new text for input;
- the submitted string for submit;
- the next checked state for a checkbox;
- the selected source index or stable option value for selection;
- the next range value and whether it is an update or commit;
- the requested open state for an overlay.

Keep existing typed action enums for advanced reducers and delegation. Build
the simple path from the same behavior rules so disabled options, keyboard
navigation and accessibility cannot diverge.

If the current `EventHandler` representation cannot carry a typed payload, do
not introduce `Any`, unchecked downcasts or a second dispatcher. Add a small,
typed runtime binding abstraction or widget/facade extension whose callback is
adapted into the existing handler registry. Preserve crate boundaries and prove
the design with input, checkbox and select before applying it everywhere.

### Interaction state policy

Review `RangeState`, `Typeahead` and similar helper state individually:

- business state such as selected value and open state remains controlled by
  the application;
- temporary gesture/typeahead bookkeeping should be retained internally when
  that can be done without hiding application-visible state;
- advanced applications must still be able to use behavior objects directly;
- do not move all widget state into the UI tree as a shortcut.

## Phase 4: create `argui-testing`

Add a focused workspace crate named `argui-testing`. It is intended primarily
as a development dependency and must not require a native window, display
server or GPU.

### Initial responsibilities

`TestApp<A>` should own or coordinate:

- an `Entity<A>` and `SingleWindowModel<A>`;
- the current `UiTree`;
- `LayoutEngine` and deterministic embedded-font `TextEngine` instances;
- a logical viewport and `WindowEnvironment`;
- the latest layout output and hit regions;
- pending application effects and a bounded settle loop.

The settle loop must render, reconcile, lay out, deliver resulting layout
changes and repeat until no work remains. It must use a documented iteration
limit and return a diagnostic error for a non-settling application rather than
hanging a test.

### Queries

Provide queries by:

- application key;
- accessible role and name;
- visible text;
- label;
- semantic state;
- focused element.

Prefer accessible queries in documentation because they verify both usability
and testability. Keys remain available for exact internal selection. Duplicate
matches must fail with a useful diagnostic rather than selecting arbitrarily.

### Input operations

The first stable release must support:

- `click` and pointer movement;
- keyboard press/release and shortcuts;
- focus, blur, Tab and Shift-Tab traversal;
- text entry, replacement and submission;
- paste and clipboard requests;
- wheel/scroll input;
- pointer drag;
- accessibility actions;
- viewport resize and environment changes.

`click` must exercise the real pointer sequence and hit testing. A separate
explicit accessibility activation helper may synthesize an accessibility
click, but it must not be the implementation of normal pointer clicking.

### Assertions and inspection

Provide clear assertions for:

- text presence and absence;
- text-input value;
- focus;
- existence and visibility;
- enabled, disabled, checked, selected, expanded, busy and invalid states;
- layout bounds;
- clipboard, focus, scroll, theme and application commands;
- pending work and final quiescence.

Action methods should return `Result` with structured errors. Convenience
assertions may panic, but the panic must include:

- the requested selector;
- zero, one or multiple matches;
- close candidate roles, labels, text and keys;
- the current focused node;
- a compact semantic/tree dump.

Never expose private engine state solely to satisfy the harness. Add public
inspection only when it represents behavior visible at a crate boundary.

### Deterministic time and tasks

After synchronous input is stable, add:

```rust,ignore
app.advance(Duration::from_millis(500));
app.run_until_idle();
```

Animations, timers and task completion in tests must not depend on wall-clock
sleep. Introduce an injectable clock/executor boundary only as far as needed by
the public harness. Preserve the normal native and WebAssembly executors.

Cover cancellation caused by unmounting, replacement, hidden presentations and
application shutdown. A dropped task must not call a stale handler.

### Later harness coverage required before completion

The plan is not complete until the harness also covers:

- overlays and modal focus;
- multiple application windows;
- application lifecycle events;
- async tasks and controlled time;
- scroll reveal and virtualized content;
- semantic/accessibility actions;
- error diagnostics for missing and duplicate selectors.

## Phase 5: migrate repository applications

Migrate examples and application code only after each corresponding widget API
is tested.

1. Convert simple local handlers in `app_examples/docs-examples`.
2. Fix the dialog example so the built trigger key and handled key cannot
   diverge, then cover opening and closing it through `argui-testing`.
3. Break the Widget Gallery's large root event router into local callbacks where
   the event is local.
4. Keep deliberate delegation for catalogue navigation, commands and other
   genuinely centralized behavior.
5. Migrate the fake AI harness and focused examples where direct callbacks
   improve clarity.
6. Add one headless behavior test for every interactive documentation example.

Do not perform mechanical migration that makes code longer. When a complex
widget is clearer with its typed behavior reducer, retain that reducer and
document it as the advanced pattern.

## Phase 6: documentation and onboarding

Reorganize public documentation around increasing complexity.

### Required learning path

1. Installation and a first window.
2. Counter using a direct button callback.
3. Controlled form with input, checkbox and submit.
4. Headless test of that form.
5. Async task and loading state.
6. Reusable child model.
7. Typed actions and shortcuts.
8. Event bubbling, capture and delegation.
9. Custom controls and low-level interaction.

Every interactive widget page must contain:

- the shortest useful example;
- its controlled state;
- its direct handler and payload;
- keyboard and accessibility behavior;
- a small `argui-testing` example;
- a link to the advanced behavior/action API when one exists.

The Events page must explicitly explain that bubbling is useful for delegation
but is not required to handle an ordinary button.

### Compile and behavior verification

- Compile every Rust documentation snippet that is intended to compile.
- Run the documentation examples through their headless tests.
- Generate the website catalogue and fail CI on duplicate slugs or stale
  generated output.
- Ensure the live gallery demonstrates the same API shown in the guides.

## Phase 7: simplify imports and feature selection

### Prelude

Add a deliberately small `argui::prelude` for application authors. Re-export
only the types required by common views and callbacks. Do not turn the prelude
into a dump of the complete public API, and do not require library authors to
use it.

At minimum, evaluate inclusion of:

- `Render`, `Context` and common application launch types;
- `Element` and common layout helpers;
- the direct callback/handler types;
- common theme types;
- widgets enabled by the application's selected features.

Check name collisions with standard-library and common ecosystem types.

### Feature profiles

Keep every granular `widget-*` feature, but add documented convenience profiles
for new applications if package-size measurements justify them. Candidate
profiles are a basic application set, desktop integrations and web. Do not make
all integrations default, and do not pull platform-only dependencies into
unrelated targets.

Measure clean compile time and stripped binary/Wasm size before and after each
profile. The simple installation path must state clearly what it enables.

### Builder consistency

Inventory widget constructors and `build()` signatures. Make the pattern
consistent where possible, but do not introduce implicit conversions that make
ownership or feature requirements unclear. Any breaking constructor cleanup
belongs in `0.3.0` and must be performed once rather than through aliases.

## Phase 8: compatibility and release preparation

1. Treat the work as the `0.3.0` API line if it includes breaking constructor,
   naming or payload changes.
2. Keep `Context::listener`, `Element::on` and behavior/action APIs because they
   are the advanced layer, not deprecated compatibility shims.
3. Remove genuinely replaced APIs and migrate all repository callers in the
   same change, following the contributor rules.
4. Add public API or semantic-version checking to CI.
5. Write a migration guide containing old and new examples for buttons, forms,
   overlays and complex controlled widgets.
6. Update `CHANGELOG.md`, the README, feature tables and crate metadata.
7. Verify all crates in a temporary registry in dependency order.
8. Verify native desktop, WebAssembly and package/archive jobs.
9. Confirm the protected branch is green before marking the plan complete.

## Test matrix

The final implementation must cover these layers independently:

| Layer | Required evidence |
| --- | --- |
| Handler registry | identity, replacement, mount isolation, routing and cleanup tests |
| Event dispatch | pointer, keyboard, accessibility, capture, bubble and cancellation tests |
| Widgets | direct handler and typed payload tests for every interactive category |
| Harness | query, action, settle-loop and diagnostic tests |
| Examples | one headless behavior test per interactive documentation example |
| WebAssembly | workspace target check with the final feature set |
| Native | hidden-display interaction tests plus existing renderer checks |
| Packaging | temporary-registry crate archive verification |
| Website | catalogue generation, snippet compilation and browser scenarios |

Each affected crate must remain above the repository's 85% floor for lines,
functions, regions and branches.

## Implementation workflow

Use small, reviewable commits or pull requests in this order:

1. Restore the green baseline.
2. Add opaque handler registration.
3. Complete Button plus the minimal harness vertical slice.
4. Complete text and boolean form controls.
5. Complete selection, range and overlay controls.
6. Complete navigation, menu and data widgets.
7. Complete the test harness, deterministic time and multi-window behavior.
8. Migrate examples, gallery and documentation.
9. Add the prelude and measured feature profiles.
10. Add compatibility checks and prepare `0.3.0`.

For each step:

1. inspect overlapping user changes before editing;
2. implement the smallest complete behavior;
3. run only directly affected crate tests with `--all-features`;
4. run Clippy for the affected crate;
5. check the affected public API on WebAssembly;
6. update documentation and tests in the same step;
7. do not leave the repository between two public API designs.

Do not repeatedly run the entire workspace during development. After all phases
are implemented, run the full quality command exactly once immediately before
the final commit, as required by the contributor guide:

```sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
```

If that command fails, fix the failure with targeted checks and then rerun the
full gate only when the implementation is again believed complete.

## Definition of done

Every item below is mandatory:

- [ ] The protected branch and release/package workflows are green.
- [ ] `EventHandler`, `Context::callback` and `Context::event_handler` are
      documented and tested.
- [ ] The low-level listener and propagation APIs retain their behavior.
- [ ] The Counter uses a handler attached directly to its button.
- [ ] A beginner can implement the Counter without `EventType`,
      `target_key()`, `UiEventKind` or explicit `cx.notify()`.
- [ ] Every interactive widget has a consistent direct API or an explicit,
      documented reason to remain action-only.
- [ ] Common widget handlers receive useful typed values rather than forcing
      event-kind matching.
- [ ] Disabled, busy and unavailable controls never call direct handlers.
- [ ] Mouse, touch, keyboard and accessibility behavior remain equivalent.
- [ ] `argui-testing` works without a native window or GPU.
- [ ] Normal `click()` uses real layout, hit testing and pointer dispatch.
- [ ] The harness supports focus, keyboard, editing, scrolling, dragging,
      accessibility, overlays, tasks, controlled time and multiple windows.
- [ ] Missing or ambiguous test selectors produce actionable diagnostics.
- [ ] Every interactive documentation example has a headless behavior test.
- [ ] The Dialog example opens and closes in its automated test.
- [ ] The Widget Gallery no longer uses one global handler for interactions that
      are naturally local.
- [ ] Deliberate bubbling/delegation remains demonstrated on the Events page.
- [ ] Public snippets compile and website catalogue generation is clean.
- [ ] The prelude and convenience feature profiles are documented and measured.
- [ ] The migration guide and `CHANGELOG.md` are complete.
- [ ] Public API/SemVer checks run in CI.
- [ ] All affected crates and the workspace meet every 85% coverage threshold.
- [ ] Native hidden-display checks and the WebAssembly check pass.
- [ ] Crates.io archives resolve only the intended staged versions.
- [ ] No placeholder, migration shim, stale caller or unfinished checklist item
      remains.
- [ ] The final quality gate passes from the exact commit intended for merge.

The work may be reported as complete only when all boxes are supported by test,
documentation or CI evidence. A partially completed phase should be reported as
progress, never as completion.
