# Widget API completion

Implementation baseline: `de335ca`. Scope agreed with the user: finish the API
contracts and the named widget families, including editable data tables. The
remaining shadcn catalog is outside this change. No compatibility layer.

## Delivery checklist

- [x] Scoped accessible relations, diagnostics and native/Web lowering.
- [x] Mixed checkbox state; migrate every consumer.
- [x] None/programmatic/tab-stop focus and PageUp/PageDown.
- [x] Stable collection identities, virtual focus and keyboard search.
- [x] Menu, ContextMenu, Menubar and nested interaction.
- [x] Controlled Toast state/host with deterministic lifetime.
- [x] Customizable Gregorian Calendar and DatePicker.
- [x] DataTable transformations, virtualization and controlled cell editing.
- [x] Focused gallery pages, public feature exports and documentation.
- [x] Behavioral and bounded-work verification.
- [x] Coverage: 85% of branches, functions, lines and regions globally and in
      every crate changed from the baseline; existing exclusions only.
- [x] Final quality gate and commit, without coverage waiver.

During implementation, run affected tests with `--all-features`. Run the final
quality script only once after implementation, then repair/recheck failed gates
if needed. Do not run concurrent LLVM coverage. All Rust files remain within
600 physical lines, and all tests mirror source paths under `tests/`.

## Current handoff

The user requested implementation and full coverage execution again on 2026-09-09.
Targeted native tests and instrumented coverage are authorized; no Chrome has been
launched. The 85% gate passes for all changed crates and the workspace. Current reports are retained in `target/coverage-report.json` and
`target/coverage-gate.json`.

Implemented API areas include scoped semantic relations and native/DOM lowering,
three-state checkboxes, focus policies, stable collections with pinned virtual
rows, typed nested menus/context menus/menubar, controlled toast lifetimes,
Gregorian calendars/date pickers, and typed editable data tables. Gallery pages
are separate widgets. The checklist above tracks accepted delivery, including
validation; it must not be checked merely because source files exist.

Recent refinements: immutable collection snapshots clone in constant time;
`Collection::remap_heights` preserves measured heights by stable identity; the
data-table page selection summary is cached; column resize handles reuse
`SplitPane`; menu events resolve their actual visible level; DatePicker events
are scoped and cancel restores the validated calendar date; initial focus
prefers Tab stops over programmatic-only elements.

### Integration contracts

- `DataTableModel::selection()` and `edit()` expose read-only state; mutations
  use `set_selection`, `begin_edit` or `apply`, so cached page summaries and
  edit validation remain consistent. `ResizeColumn` reuses the existing
  `SplitPane` gesture/keyboard behavior. The gallery uses variable row heights
  so validation messages can increase an edited row's height.
- `Menu::submenu_content_key` exposes the geometry anchor for `MenuIntent`.
  Gallery layout delivery supplies its bounds, pointer movement updates the
  corridor deadline, and changing/closing the menu cancels the task.
- `CalendarLocale::rtl` controls horizontal date navigation. `DatePicker` scopes
  events to its own keys, associates open popup content with the input and
  restores the validated calendar state on cancellation.
- Initial focus first considers enabled Tab stops, then programmatic targets.
  Explicit focus requests continue to accept either focusable policy.

### Final integration work

- Menus show checkbox/mixed/radio indicators and directional submenu arrows.
  Arrow keys open first/last entries; a closed menubar moves focus without opening.
  Tab dismisses a menu and follows normal focus navigation. Closing a nonmodal
  scope preserves focus that has already moved outside it. Delayed hover is
  cancelled on keyboard actions; pointer movement updates the submenu corridor.
- TreeView uses one Tab stop and pins its active row, with and without row caching.
  Tests exercise focus across a virtual scroll and Tab leaving the collection.
- DataTable shares a horizontal viewport between headers and rows. A layout test
  scrolls an overflowing table before and after column resizing and compares the
  resulting header/cell positions.
- Deterministic gallery tests exercise menus, dates, notifications, data-table
  editing, action menus and task cancellation.
- GTK now translates PageUp/PageDown, ContextMenu and function keys, matching
  the winit adapter. Native input tests send GTK signals to the lifecycle test's
  own window and assert translated key presses/releases, mouse button masks and
  scroll direction. No desktop-wide input injection is used.
- Native lifecycle progress is driven by model task completions, independently
  of redraw callbacks. Scroll/focus requests are forwarded by the test's composed
  window model. The scenario checks hide/show mount retention, close/reopen
  identity, scrolling and cancellation of both view-owned and model-owned work.

### Verification snapshot — 2026-09-09

The complete instrumented suite passed **958 tests** with one worker. Targeted
runtime/UI/widget/gallery regressions passed 587 tests before that measurement.
The coverage aggregation has five passing Python regression tests. The final
measurement started from clean workspace instrumentation with unchanged feature
flags and OS/GPU exclusions.

| Scope | Branches | Functions | Lines | Regions |
| --- | ---: | ---: | ---: | ---: |
| Workspace | 88.72% | 92.81% | 93.64% | 93.15% |
| Widgets | 88.49% | 94.43% | 95.51% | 94.68% |
| Widget gallery | 85.57% | 92.08% | 94.94% | 94.78% |
| Runtime | 85.09% | 85.52% | 87.09% | 86.57% |

All changed crates pass their applicable metrics. The detailed export unions
source branch outcomes across generic instantiations when every branch location
is accounted for; denominators and exclusions remain unchanged. See
[code quality](code_quality.md) for the conservative treatment of folded
expressions and macro expansions. No coverage waiver is used.

The final runtime work also fixes inspector-cache invalidation when text privacy
changes and prevents retired handler slots from being reused. Continuous handler
slots remain stable across renders and hide/show. Tests cover stale deliveries,
private text snapshots, model-scoped tasks, closing during updates, dropped
mounts, multiple cleanup panics and final lifecycle delivery during unwinding.
Native tests verify edit commands and theme propagation as well as input and
window lifecycle.

Structure, Rust size, test layout, hygiene, formatting, workspace native Clippy,
wasm compilation and model-context doctests passed. Runtime Clippy and wasm
compilation were repeated after runtime edits. `quality.sh` was invoked once;
its failed native check was repaired and the constituent checks were rerun
separately, including the complete instrumented suite above.

Native validation used sequential GTK/Wayland runs. Earlier native runs had
intermittent Wayland protocol/segmentation faults; this work does not claim to
resolve every compositor/driver failure. X11 cannot validate the Wayland-only GTK
canvas. Browser and real screen-reader audits were not run; no Chrome was launched.

The current coverage gate is `target/coverage-gate.json`; the measurement is
`target/coverage-report.json`. Reproduce the full native measurement with
`ARGUI_NATIVE_TESTS=1 ARGUI_COVERAGE_BASE=de335ca ./scripts/check-coverage.sh`.
