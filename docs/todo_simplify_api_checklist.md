# Simplified API implementation checklist

This checklist tracks the evidence required by
[`todo_simplify_api.md`](todo_simplify_api.md). A box is checked only after the
implementation, its tests, and the relevant documentation or CI evidence are
present. Remote-only checks remain open until the exact merge commit has run on
the protected branch.

## Phase 0 — green baseline

- [x] Record the failing and passing jobs from the latest protected-branch run.
- [x] Remove every duplicate website catalogue slug and regenerate the catalogue.
- [x] Make archive verification resolve staged workspace crates only.
- [x] Split or optimize the quality workflow so no job reaches its timeout.
- [x] Run the affected website, release-script, and workflow checks.
- [x] Record the baseline test count and per-crate/workspace coverage.
- [ ] Confirm the protected-branch workflow is green.

## Phase 1 — handler foundation

- [x] Add and document opaque `argui_ui::EventHandler` identity.
- [x] Bind handlers to event types without exposing owner/slot construction.
- [x] Model and test direct-target delivery across capture, target, and bubble phases.
- [x] Re-export the handler from `argui-ui` and the facade.
- [x] Share one runtime registration primitive between all callback APIs.
- [x] Add and document `Context::callback` with implicit invalidation.
- [x] Add and document `Context::event_handler` without implicit invalidation.
- [x] Reimplement `Context::listener` through the shared primitive.
- [x] Test stable identity, replacement, removal, routing, mount isolation, visibility,
      closure, propagation, once, and passive semantics.
- [x] Pass targeted `argui-ui` and `argui-runtime` tests and Clippy.

## Phase 2 — Button vertical slice

- [x] Add documented, additive `Button::on_click(EventHandler)`.
- [x] Attach direct handlers to the interactive root built by `ButtonBehavior`.
- [x] Test pointer, touch, keyboard, accessibility, disabled, busy, custom-content,
      multiple-handler, bubbling, cancellation, and stale-replacement behavior.
- [x] Rewrite the Counter example and add its headless behavior test.
- [x] Keep Events as the explicit delegation example.
- [x] Add the direct-handler Button catalogue example.
- [x] Pass the relevant WebAssembly check.

## Phase 3 — consistent widget APIs

- [x] Inventory every exported widget as interactive or intentionally presentational.
- [x] Add typed handlers for `Input`, `TextArea`, and `InputOtp`.
- [x] Add typed handlers for `Checkbox`, `Switch`, `Toggle`, and `ToggleGroup`.
- [x] Add typed handlers for `RadioGroup`, `Tabs`, `Select`, `NativeSelect`, and `Combobox`.
- [x] Add typed handlers for `Range`, `Slider`, and `ColorPicker`.
- [x] Add typed handlers for collapsible, accordion, dialog, alert-dialog, sheet,
      drawer, popover, and hover-card controls.
- [x] Add typed handlers for menu, context-menu, menubar, and command-palette controls.
- [x] Add typed handlers for calendar, date-picker, pagination, carousel, and breadcrumb.
- [x] Add typed handlers for list, VList, tree-view, table, and data-table controls.
- [x] Add typed handlers for interactive composites, including questionnaire,
      message-scroller, toast, updater, and sidebar controls.
- [x] Preserve controlled business state and retain only temporary interaction state.
- [x] Test typed payloads and disabled/busy/unavailable behavior for every category.
- [x] Document an explicit reason for every action-only or presentational widget.

## Phase 4 — `argui-testing`

- [x] Add the publishable `argui-testing` workspace crate and facade documentation.
- [x] Implement a bounded render/reconcile/layout/event settle loop with diagnostics.
- [x] Implement unique queries by key, role/name, text, label, semantic state, and focus.
- [x] Implement pointer click/move, real hit testing, keyboard, shortcuts, focus,
      Tab traversal, text editing, paste/clipboard, scrolling, drag, accessibility,
      resize, and environment changes.
- [x] Implement assertions for text, value, focus, visibility, semantic states, bounds,
      effects, pending work, and quiescence.
- [x] Include selector candidates, focus, and a compact semantic/tree dump in errors.
- [x] Implement deterministic time, task completion, cancellation, and idle draining.
- [x] Cover overlays/modal focus, multiple windows, lifecycle, virtualization/scroll
      reveal, accessibility actions, tasks, time, and selector diagnostics.
- [x] Prove the crate needs no native window, display server, or GPU.

## Phase 5 — repository applications

- [x] Migrate simple handlers in `app_examples/docs-examples`.
- [x] Fix the dialog trigger-key mismatch and test open/close headlessly.
- [x] Replace naturally local Widget Gallery routing with local callbacks.
- [x] Preserve deliberate navigation, command, and delegation routers.
- [x] Migrate the fake-AI harness and focused examples where clearer.
- [x] Add one headless behavior test per interactive documentation example.

## Phase 6 — documentation and onboarding

- [x] Document installation and the first window.
- [x] Document the direct-callback Counter.
- [x] Document a controlled form and its headless test.
- [x] Document async/loading, reusable children, typed actions/shortcuts, delegation,
      and custom low-level controls in increasing-complexity order.
- [x] Give every interactive widget page a short controlled example, handler/payload,
      keyboard/accessibility behavior, harness example, and advanced API link.
- [x] Explain on the Events page that ordinary buttons do not require bubbling.
- [x] Compile all intended Rust snippets and run their headless behavior tests.
- [x] Regenerate the unique, current website catalogue and verify browser scenarios.
- [x] Keep the live gallery API aligned with the guides.

## Phase 7 — imports, features, and builders

- [x] Add and document a deliberately small `argui::prelude`.
- [x] Check prelude name collisions and feature-gated widget exports.
- [x] Measure clean compile time and stripped native/Wasm size before profiles.
- [x] Add only justified basic/desktop/web convenience feature profiles.
- [x] Measure and document compile time and native/Wasm size after profiles.
- [x] Inventory constructors and `build()` signatures and complete the one-time 0.3 cleanup.

## Phase 8 — compatibility and release preparation

- [x] Move the workspace to the `0.3.0` API line.
- [x] Preserve `Context::listener`, `Element::on`, and typed behavior/action APIs.
- [x] Remove replaced APIs and migrate every repository caller without shims.
- [x] Add public API/SemVer checking to CI.
- [x] Write the button, form, overlay, and controlled-widget migration guide.
- [x] Update `CHANGELOG.md`, README, feature tables, and crate metadata.
- [x] Verify every crate archive in dependency order against a temporary registry.
- [x] Verify native desktop, WebAssembly, mobile, package, and website jobs.
- [ ] Confirm the protected branch is green.

## Final acceptance evidence

- [x] Handler registry, event dispatch, widgets, harness, and examples have the full
      independent test coverage listed in the plan.
- [x] Every affected crate and the workspace exceed 85% for lines, functions,
      regions, and branches.
- [x] Native checks pass on the hidden display and captures are non-blank.
- [x] The final WebAssembly workspace check passes.
- [x] No `TODO`, `FIXME`, placeholder, compatibility shim, stale caller, or unchecked
      implementation item remains.
- [x] The exact final commit passes
      `./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh`.

## Recorded evidence

- Protected-branch run 35016485097 failed in the archive and website jobs and
  cancelled the monolithic quality job after 87 minutes; security, both desktop
  checks, mobile cross-check, Android packaging, and iOS packaging passed.
- The baseline report recorded 720 passing tests. Its workspace coverage was
  96.01% lines, 95.52% functions, 95.42% regions, and 89.36% branches; every
  measured crate met the 85% gate.
- Targeted API verification passed 838 tests; the headless harness passed 18
  tests and all 14 interactive documentation examples passed independently.
- Android and iOS cross-checks, the complete native and Wasm example workspaces,
  all 24 staged crates.io archives, and 111 generated website routes passed.
- The hidden-display browser suite passed its desktop and mobile flows, themes,
  search, routing, copy, live Wasm interaction, fallback, 404, and hydration checks.
- The private Wayland capture was 1600×1200 and 643,396 bytes with visible
  gallery content; it was not a blank-frame acceptance.
- The final protected-branch boxes remain open until this branch is pushed and
  the exact commit has completed remote CI.
