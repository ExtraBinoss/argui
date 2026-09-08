# Shared state and custom elements — implementation record

> Mise à jour du 8 septembre 2026 : les passages et commandes concernant
> Shared Mailbox, Draft Studio et GalleryApplication ci-dessous sont historiques.
> Ces démonstrations et leurs tests dédiés ont été supprimés lors du recentrage
> de la galerie sur les widgets. Les contrats de modèles/montages restent testés
> dans `argui-runtime/tests/model/`. Le lanceur utilise `SingleWindowModel`.
> Les anciennes preuves de galerie ne sont plus des vérifications exécutables.


This work implements the two contracts planned after `oberservations.md`:
shared models/lifecycles and publicly extensible elements. The full scope below
remains the acceptance target; intermediate tests do not establish completion.

## Current implementation

For point 1, use the [dated evidence and closure checklist](point-1-completion.md).
It supersedes historical remaining-work lists below for lifecycle status and
separates tested contracts, missing integration proofs and manual scenarios.

### Custom-container completion update

The earlier chronological notes below about leaf-only elements are superseded:
`CustomLayoutContext` now measures and places ordinary children through the retained
engine; `Element::custom_region` declares keyed interactive/semantic children in
the same UI tree. The gallery's ruler is a custom container, with ordinary clip
buttons and accessible resize handles. Its two presentations share one clip model
without sharing selection, zoom or scroll. See [the current contract](custom-elements.md).

Targeted UI/layout/gallery tests passed 374 cases across 46 binaries; the affected
runtime inspection suite passed another 17 tests. They include invalid placement recovery, region identity rejection,
clipped/transformed hit testing, capture outside bounds, removal during drag,
semantic removal, shared edits, keyboard/semantic resizing and paint-only phase
counters. A 1,000-cycle test verifies keyed reordering preserves custom state and
removal releases every instance and retained layout node. The Web script passed
shared semantic resizing, captured pointer resizing outside handle bounds,
independent zoom and unmount without console errors. Its software-Chromium screenshot remained white;
visual correctness is not inferred from that run. Final coverage has not been run.

This update concerns E–I and the three explicitly requested missing features;
it does not retroactively certify every earlier lifecycle requirement in A–D.

The final grouped-timeline Web run exposed an adapter bug: nested semantic DOM
nodes used surface coordinates as parent-relative coordinates. `SemanticTree::
parent_relative_bounds` now supplies the actual parent-local positions, including
reparenting and parent-only movement. Layout also records transformed/clipped
semantic bounds with its paint fragments, so cached painting preserves the same
geometry used by accessibility. Ten accessibility tests and 21 targeted
layout/paint tests passed; the rebuilt browser scenario then passed real captured
dragging with semantic groups retained. Targeted Clippy, formatting and source
structure checks passed. No commit or global coverage run was performed.

- Basic `Entity<T>` creation, identity, weak references, reads and updates no
  longer require `Render`. Owned task spawning also accepts data-only models.
- `ResourceScope` and non-owning `ResourceLease` provide explicit, idempotent
  cleanup. Entity destruction closes its scope even when callers retain scope
  clones. Subscriptions register against both endpoint scopes; cancellation
  removes their registrations immediately.
- Tasks can additionally belong to an explicit scope. Completion releases scope
  registrations before calling user code without marking successful work cancelled.
  Cancellation through a handle or either owner, and runtime shutdown, release
  all task scope registrations even when the task handle remains alive.
- `EntityId` is process-unique and does not reuse a dropped entity's address.
- `ModelRuntime` owns a UI-thread transaction/event queue. Context-created data
  entities share that runtime. Event callbacks run after model borrows end.
- The attached task executor now belongs to `ModelRuntime`, not each entity.
  Models created before attachment and data-only models created through context
  share it without rendering or retaining their creator. Task cancellation still
  follows each model's resource ownership. Presentation-tree attachment and host
  shutdown policies remain to be migrated to application/mount ownership.
- `EventEmitter<E>`, `Context::emit`, `Context::subscribe` and RAII
  `Subscription` provide typed, ordered delivery with weak endpoints.
  Cancelling an endpoint or subscription suppresses queued callbacks.
- Event delivery and dependency propagation share a budget of 64 work steps
  per dispatch. A step visits one event listener or one invalidation edge;
  large subscriber sets cannot bypass the budget. Event capacity is 1024,
  counting the event currently delivering, with explicit capacity errors.
  A host wake callback can
  schedule remaining work. Native/Web host event branches now handle deferred
  model work; full application/mount integration is not yet complete.
- A model runtime can receive its host wake after models have been created.
  Attaching or replacing the callback schedules existing pending work, while
  idle attachment does not wake. Automatic transaction drains preserve the
  outstanding-wake flag; only an explicit host dispatch acknowledges it, so
  repeated transactions do not flood the event loop with duplicate wakes.
- `ModelsReady` now connects the dispatcher to Winit/Web and GTK hosts. Attached
  rendered/routed children inherit the host callback. A host turn visits each
  encountered model runtime once across windows and forwards pending model UI
  effects through the same application path as task completions. This still
  traverses presentation entities: separately owned observed runtimes, per-mount
  effects/caches and application ownership remain migration work.
- `Context::observe` invalidates a dependent cache; `Context::read` retains a
  dependency for that render. Rebuilding without the read detaches it.
- Cached child dependencies now use detachable subscriptions instead of an
  accumulating map keyed by parent addresses. Invalidation reaches cached
  ancestors. A dirty cached root contributes a rebuild when its effects drain.
- Invalidations coalesce per outer transaction and traverse dependencies
  iteratively. Each propagation wave tracks visited entities to terminate
  cycles; subsequent transactions have separate waves so a view repainted
  during old work is still invalidated by a newer modification. Queued work
  holds weak cache references and releases completed wave bookkeeping.

## Remaining requirements — shared models and lifecycle

- Separate model context services from window/presentation services. Their
  method availability is not yet the final context contract.
- Integrate the model dispatcher with real application/window wakes, including
  task callbacks and shared application models. Do not strand deferred work.
- Integrate the existing entity/resource scopes with application and mount ownership.
- Distinguish creation, mount, visibility, unmount, retention and destruction;
  deliver lifecycle hooks outside layout/paint calculations.
- Isolate caches, handlers, layout, focus and presentation state per actual
  window/mount. Sharing data must not share presentation coordinates.
- Make application/entity/mount tasks follow their explicit owner. The old
  subtree task cancellation and executor attachment semantics still need
  replacement; closing one view must not cancel a surviving shared model.
- Integrate detachable subscriptions and resource cleanup with mounting.
- Migrate runtime hosts, adapters, SelectionHost, DevTools and gallery users;
  remove superseded paths rather than retaining compatibility wrappers.
- Add Shared Mailbox gallery example with data-only model, unread counter,
  list/detail, a second native window, unmount/remount and real task lifetimes.
  Web demonstrations must use actual backend capabilities, not fake windows.

## Remaining requirements — custom elements

- Public generic element extension with typed properties and retained state,
  stable instance identity and explicit revision/invalidation semantics.
- Leaf and container measurement/layout, constraints, baselines, child placement,
  scrolling, transforms, DPI, invalid measurements and convergence handling.
- Geometry preparation and renderer-neutral paint context using the existing
  text, vector, image, clip, layer and effect pipelines and asset contracts.
- Stable interactive subregions using existing event routing, pointer capture,
  gestures, focus, keyboard and action resolution.
- Accessible semantic subtrees and cleanup when an element or subregion is removed.
- Independent layout, interaction, paint and semantic invalidation; retained
  fast paths, no idle animation loop and DevTools visibility.
- Custom Timeline implemented in the gallery crate using public APIs only:
  draggable/resizable blocks, text, standard widget children, zoom/scroll,
  keyboard/actions, semantics, light/dark/reduced-motion and real measurements.
- Combined example showing the same model in independent presentations.

## Completion evidence required

- Adversarial lifecycle, reentrancy, cancellation, stale-result, identity,
  multi-window and cleanup tests in source-mirrored `tests/` files.
- Public-API integration tests, layout/paint/hit/semantic tests and native/Web
  scenario validation. Report real visual checks separately from compilation.
- Measured idle work and resource stability after repeated mount/unmount.
- Every old consumer migrated; contracts and examples documented accurately.
- Rust files at most 600 lines. No speculative dependencies or DSL coupling.
- Targeted `cargo nextest run --all-features` during work; no concurrent LLVM
  coverage. Run the full quality gate only after the entire implementation is
  complete and immediately before an authorized commit.
- At least 90% independently on branches, functions, lines and regions. The
  previous audit's below-threshold workspace coverage is still outstanding.

## Checks during the first implementation step

The runtime suite reached 65 passing tests after model/event/cache changes.
Tests exercise data-only tasks, event ordering, reentrant model updates,
subscription cancellation/destruction, bounded event feedback, queue capacity,
tracked read detachment, transitive invalidation and host effect draining.
The dependent suite (`argui-runtime`, `argui-widgets`, `argui-widget-gallery`,
`argui-devtools`, all features) passes 225 tests. Runtime all-target/all-feature
compilation for wasm32 also passes. Source size, test layout, source hygiene,
formatting and diff whitespace checks pass.
These results prove this step only, not the remaining lifecycle or custom-element work.

## Second implementation step

The runtime suite reached 70 tests after bounded/coalesced invalidation and
per-listener event delivery. New adversarial cases cover 128 observing views,
20 notifications in one transaction, a 1024-node dependency cycle, a new
transaction overlapping an older propagation wave, a 130-listener event,
FIFO ordering across event batches, and capacity admission during delivery.
The final dependent suite passes 230 tests across 37 binaries, with no skips.
Runtime Clippy (all targets/all features) and wasm32 compilation (all targets/
all features) pass, as do the source structure and whitespace checks.

## Resource lifetime step

The runtime suite now passes 81 tests across six binaries, without skips.
Additional cases exercise scope cleanup order, idempotence, reentrant cleanup,
cleanup panics, closed scopes, retained handles after owner destruction, 1,000
subscription/cancellation cycles, selective task cancellation, completion-time
cleanup before callbacks and cleanup during runtime shutdown. This is resource
ownership evidence, not proof of application/window/mount integration, which
remains required above. The implementation sequence and full acceptance criteria
are recorded in [the plan](shared-state-custom-elements-plan.md).

## Shared executor step

Moved executor storage from per-entity task owners to the shared model runtime.
A public-API test creates data models before and after executor attachment,
drops their creator, and delivers both task results without rendering either
model. The four-crate dependent suite passes 242 tests across 37 binaries with
no skips. Runtime all-target/all-feature Clippy and wasm32 compilation pass;
source size, test layout, source hygiene and whitespace checks pass. Real model
dispatcher wakes and independent presentation mounts remain outstanding.

## Dispatcher wake preparation

The runtime suite passes 84 tests, including late host attachment, host callback
replacement, coalescing across ten intervening model transactions, host wake
acknowledgment and idle behavior. This prepares the dispatcher contract; the
native/Web host event handlers still need to attach and drain it, and shared
model changes still need to reach independently owned presentation mounts.

## Host event connection

Added native/Web `ModelsReady` handling and shared GTK/Winit multi-window update
orchestration. Model effects and task effects now share application-side UI
application code. Public tests verify rendered and routed children inherit the
host callback without redundant wake rebinding on render. The dependent suite
reaches 245 passing tests across 37 binaries without skips. Runtime Clippy and
wasm32 all-target/all-feature checks pass, as do source structure checks. These
are compilation and deterministic logic checks, not an interactive window test.
Independent mounts, cross-window effect ownership and the remaining custom
element requirements are not complete.

## Host dispatch ordering

Host processing now separates runtime discovery, bounded event dispatch and
effect collection. Multi-window hosts dispatch all discovered runtime batches
before collecting any window's effects, so a later runtime's callback can update
an earlier window before its effects are consumed. Single-window hosts use the
same ordering across rendered and routed descendants. Runtime discovery invokes
no user callbacks while borrowing the retained child lists.
The runtime regression suite passes 85 tests without skips. This ordering change
still needs direct host scenario coverage and does not establish independent
per-mount caches/effects or complete application lifecycle behavior.

## Closed event owners

Emission now rejects a closed entity scope with `EventError::ScopeClosed`, even
if all listeners were already removed. Queued events detect a closed source and
discard its remaining listener snapshot in one work step, releasing the event
payload without spending one step per cancelled listener. Attaching an inactive
subscription to a closed scope also rejects the operation consistently with tasks.
The runtime suite passes 88 tests, including a queued payload with 200 listeners
whose source closes before delivery; no callback runs and the payload is released
in the first bounded drain. These are entity-scope guarantees; application and
mount lifecycle integration remain outstanding.

## Presentation state boundary

Retained cache, pending UI effects, child presentations, event routes, window
environment and handlers now live in `model/presentation.rs`, separate from the
model value, typed event registry and entity resource ownership. All rendering,
handler, task and host traversal consumers use this boundary. This is a structural
migration with unchanged behavior, not the final mount implementation: an entity
still owns one presentation, and model invalidation still uses its render cache.
Those two remaining couplings must be replaced before claiming multi-mount
isolation. The runtime regression suite remains at 88 passing tests without skips.

## Model invalidation boundary

The follow-up split moves observer registration and bounded dependency propagation
into `model/signal.rs`, owned by the entity rather than its presentation cache.
`Entity::revision()` exposes the invalidation revision: explicit notifications
and dependency invalidation advance it, rendering and paint-only requests do not.
Each render cache now records the revision it rendered instead of clearing a
shared dirty flag. Thus cache completion cannot reset the model's invalidation
history. Two public tests cover a data-only model with 100 notifications and a
rendered view whose cache reads leave the revision unchanged. Independent mount
identity, storage, lifecycle and per-mount dependencies remain to be implemented.

## Presentation handler identity

Handler ownership now uses an internal `PresentationId` rather than the model's
`EntityId`. Both draw from a non-reusing identity allocator, but remain distinct
typed identities. Render context listener creation and event routing use the
presentation identity consistently. A public UI-event test verifies that the
owner differs from the model identity, survives re-rendering, and rejects an
old presentation's delivery after its model is destroyed and replaced.
The runtime suite passes 91 tests without skips. This is the identity foundation,
not the public multi-mount contract: each entity still owns one presentation.

## Shared model ownership boundary

Entity storage now separates a reference-counted `ModelState<T>` from its
presentation. Model identity, data, invalidation, event registry, resources and
task ownership reside in `ModelState`; its destruction closes the entity resource
scope. Presentation handlers/cache/environment are not part of that state.
All existing consumers have been migrated to the explicit model field (no
forwarding compatibility layer). Runtime regression tests remain at 91 passing
tests without skips. Public independent mount creation and weak model/mount
lifetimes are still required; this storage split alone does not deliver them.

## Explicit root mounts

`Entity::mount()` now creates a fallible `Mount<T>` with an independent retained
presentation over the same `ModelState<T>`. `MountId`, `WeakMount`, rendering,
event dispatch and a presentation resource scope are public. Cloning a mount
retains its identity; creating another mount allocates a new identity. Closing
the model scope closes mount scopes; closed mounts reject rendering and event
dispatch. Dropping a mount closes its scope even if a caller retained a scope
clone, without destroying a model used by another mount.

`WeakEntity` now tracks the shared model as well as its originating presentation:
if that presentation has gone but the model remains, upgrading returns a fresh
entity presentation. `WeakMount` never resurrects a dead presentation. Public
tests cover independent light/dark caches, owner-filtered event dispatch, shared
data invalidation, weak lifetime separation, new mount identity and closed scopes.
The dependent suite passes 254 tests across 37 binaries without skips; runtime
Clippy and wasm32 compilation pass with all targets/features, as do structural
checks. Native interactive scenarios have not been run.

This is root-mount support, not completion of the mount contract. Child mounting
through `Context::entity`, per-mount dependency invalidation, lifecycle hooks,
presentation-owned task/context APIs and host/wrapper migrations remain required.
Existing direct entity rendering must be migrated, not retained as a permanent
parallel compatibility API. Shared Mailbox and custom element examples also remain.

## Child mount ownership

`Context::entity` now creates a presentation owned by the current parent and
reuses an existing child presentation by model identity and occurrence. Separate
parents do not share child caches, environments or handler owners. Removed child
presentations release their model/parent resource registrations outside the
retained-list borrow. A detached context can no longer create an ownerless child
view; its previous test has been migrated to the explicit rejection contract.
Parent scope closure also closes child scopes. Layout forwarding now targets
the current parent's retained presentations, and `Mount::layout_changed` rejects
closed mounts.

Application commands now reside with the shared model and drain once, fixing a
regression where a data-model task's command remained in its original presentation
instead of reaching the mounted view. Presentation-local effects remain separate.
The nested mount test covers light/dark children, retained-cache reuse, isolated
event owners, layout environment routing, removal and model survival. Host/root
migration, per-mount dependency tracking, full lifecycle hooks and explicit scoped
task/context APIs remain outstanding, along with the examples and custom elements.

## Mount-local tracked invalidation

Each render cache now has a local invalidation signal in addition to the shared
model revision. Render-scoped tracked reads target that local signal and attach
their subscriptions to the presentation resource scope. Parent render caches
observe both child model notifications and child-local dependency invalidation.
Explicit model observation remains model-wide. Host/task effect aggregation marks
the presentation cache, rather than falsely notifying the shared model.
A public two-mount test verifies independent source reads, unchanged cache identity
in the unaffected mount, unchanged shared-model revision and subscription cleanup
when the affected mount is dropped. The dependent suite reaches 257 passing tests
across 37 binaries. Remaining host/lifecycle/context migrations and custom element
work are still required; these tests are not native interactive validation.

## Host root ownership and cancellation migration

`SingleWindowModel` now retains an explicit mount, and multi-window adapters are
created as mounted roots. Application teardown closes root presentation resources
instead of cancelling tasks recursively through rendered entities. Removed the
old `cancel_tasks` path and duplicate `TaskOwner` registry: task cancellation now
uses resource scopes and runtime shutdown only. The task guide reflects this
change. A real mount lifetime test verifies local cancellation on mount drop,
shared work survival after dropping its original entity handle, and cancellation
only when the last mount releases the model. The final runtime suite passes 98
tests; Clippy, wasm32 and structural checks pass. The dependent suite passed 257
tests after the initial host migration, before the final registry removal/test.
Routed external presentations, full lifecycle notifications, context separation,
gallery examples and custom elements remain outstanding.

## Mount-owned task entry points

Added `Mount::spawn` and `Mount::spawn_latest`, using the existing bounded task
executor, both owner scopes and a `WeakMount` delivery target. Unlike weak model
delivery, this path cannot recreate a presentation after it was destroyed.
Callbacks use the exact mount's context/environment. Tests cover completion
scope cleanup, presentation environment, destruction after queueing, replacement,
missing executor and closed scopes. The runtime suite passes 101 tests without
skips. This advances scoped task ownership; full model/view context separation,
lifecycle notifications, application services and custom element work remain.

## Immediate mount closure

`Mount::close` closes its presentation even while strong handles remain. Mount
scope cleanup now releases cache/dependencies, handlers, children, routes, pending
UI effects and parent/model registrations; model-scope and parent-scope closure
use the same cleanup path. Closed presentations reject routed handlers and do
not accumulate new local effects after an in-flight callback closes them.
Event routing clones the target outside the retained-child registry borrow before
calling application code. Tests verify retained closed handles, child/model
registration release and a child callback closing its own parent without a borrow
panic. Task cleanup tests compare against the mount's live-resource baseline,
which now includes its own cleanup registration. The runtime suite passes 103
tests without skips. Full lifecycle notifications and the remaining plan are not
complete.

## Mount-owned typed subscriptions

`Mount::subscribe` registers typed events against the exact presentation and its
resource scope. Closing or dropping that mount detaches the subscription and
suppresses queued delivery while leaving sibling mounts subscribed. It preserves
the existing cross-runtime rejection and source/model lifetime constraints.
`Entity::update` now returns the closure's result, so registration and other
fallible model operations can return their typed outcome directly without an
external mutable result slot. Tests cover light/dark presentation contexts,
closure while an event is queued, retained subscription handles and rejection
of endpoints from different runtimes without registration leaks. The dependent
suite passes 265 tests across 37 binaries without skips. The complete lifecycle,
context separation, gallery and custom-element work remain outstanding.

## Context method boundary

Presentation operations on `Context<T>` now require `T: Render`: environment,
paint/animation requests, pointer capture, clipboard, scrolling, focus, selection,
theme, child rendering/layout and UI editing/action commands. Data-only models
retain entity creation, notification, typed events, tasks and application commands.
No fake `Render` implementations were added to data models to bypass this boundary.
The dependent suite remains at 265 passing tests. This is a method-availability
restriction, not completion of the planned separate model/view context types;
window services still need an explicit live-mount context rather than just a
`Render` bound. The remaining lifecycle, gallery and custom-element work is intact.

## Model context consumer migration (in progress)

`Entity::update` now exposes `ModelContext`, while `Mount::update` exposes the
presentation context and rejects a closed mount before invoking user code.
Runtime task tests use the model context; paint-only revision tests use a retained
mount. A regression test checks different light/dark mount environments, closure
rejection and continued updates through a surviving mount of the same model.
Targeted validation: `cargo nextest run -p argui-runtime --all-features`
passes 106 tests across six binaries with no skips.

This does not establish a compiling consumer migration: DevTools' standalone
layout path and selection-host animation tests still need migration to retained
presentations. The backing model/presentation storage also remains to be fully
separated. No claim of workspace validation or completion of lots A–I is made.

### Selection-host consumers migrated

Selection-host tests now retain `Mount` handles for rendering, event dispatch
and animation updates, checking the fallible mount operations instead of invoking
view callbacks through `Entity::update`. `Mount::read` reads the retained shared
model without granting presentation services. Host-specific tests are grouped
under `tests/text_selection/host.rs`, keeping each Rust file below 600 lines.
The reduced-motion command test explicitly keeps its window environment while
dispatching the command instead of accidentally restoring default animations.

Validation: 193 runtime/widgets tests across 25 binaries pass with all features;
targeted all-target Clippy passes with warnings denied. Rust size, test layout,
source hygiene and diff whitespace checks pass. DevTools' standalone layout
consumer is still pending, as are the remaining plan lots and global validation.

### DevTools layout ownership

Removed the standalone child-layout call through `Entity::update`.
`inspect_layout` updates inspector geometry only; the `Render::layout_changed`
implementation delivers the adjusted application viewport through
`Context::layout_entity`, targeting the actual child of that parent mount.
Inspector-only tests and the profiling example use the geometry-only API.
A regression mounts the same DevTools model twice, checks light/dark layout
delivery and adjusted bounds independently, then verifies rejection after closure.

All 44 DevTools tests and targeted all-target Clippy pass with all features.
Source size, test layout, hygiene and diff checks pass. The gallery test build
now exposes six model-context/view-context mismatches in animation and liquid-glass
layout tests; those consumers remain to migrate. Standalone rendering and the
underlying model/presentation storage are not yet the final architecture.

### Gallery view-context consumers

Gallery interaction and liquid-glass tests now retain mounts for their view
operations. The six reported context mismatches are resolved without granting
window capabilities to `ModelContext`. `Mount::animation_frame` delivers a frame
through the live presentation context; gallery animation tests use that entry
point. A runtime regression verifies that closed mounts reject frames and a
surviving mount receives its own environment.

Runtime/gallery validation passes 136 tests across 12 binaries with all features.
Targeted all-target Clippy with warnings denied, source size, test layout, hygiene
and diff checks pass. This is consumer migration, not completion of the model
storage split, lifecycle hooks, shared-mailbox example or custom-element API.

### Subscription ownership follows the context

Subscriptions registered through a view `Context` now attach to the current
presentation scope as well as both model owners. Closing that mount cancels
queued callbacks even when application code retains the subscription handle and
the shared model. `ModelContext` subscriptions remain model-owned.
Likewise, view observations invalidate the local presentation cache, not the
shared model signal, and detach at unmount. Model observations continue to
invalidate the shared model. The registration implementation is shared; there
is no separate event dispatcher or compatibility entry point.

Two regressions exercise queued delivery during close, surviving model-owned
subscriptions, isolated cache invalidation and retained subscription handles.
All 269 tests across runtime, widgets, DevTools and gallery pass (37 binaries,
all features, no skips). Targeted runtime Clippy and structural checks pass.
Context task callbacks still require the corresponding ownership audit; storage
separation and the remaining plan deliverables are unfinished.

### Task ownership follows the context

`Context::spawn`, its explicit-scope/latest variants and blocking work now bind
delivery to the calling presentation as well as the shared model. `ModelContext`
launches remain model-owned; adding an explicit scope only narrows that lifetime.
`Mount::spawn` uses the same presentation launch path rather than duplicating
executor registration. Cancellation of a view cannot cancel an independent
model-owned task. The public task contract in `tasks.md` reflects these semantics.

Regression tests cover a queued view result during close, a surviving model task
and cancellation reaching an already-started cooperative blocking worker. All
271 runtime/widgets/DevTools/gallery tests pass across 37 binaries with all
features and no skips. Runtime Clippy and structural checks pass. The storage
split, complete lifecycle, examples and custom-element pipeline remain open.

### Model mutations do not accumulate presentation effects

`Entity::update` now enters a model-only mutation path: it does not select a
presentation owner or window environment and commits only model commands and
model invalidation. Model-owned event and task callbacks use this same path;
presentation-owned callbacks continue to use the live view path. Thus a model
notification no longer also accumulates a presentation-local pending update.
This is an internal effect-boundary separation; `ModelContext` still projects
over internal context storage and entity/presentation allocation remains coupled.

The 271 dependent tests pass with all features; runtime Clippy, the runtime
all-target wasm32 check and structural checks pass. No complete-plan or browser
interaction validation is claimed.

### Explicit model domains for observation

Explicit observation now returns `DifferentRuntime` before registration when
endpoints belong to independent runtimes; render-tracked reads diagnose the same
contract violation before installing a dependency. Tests check rejection and
absence of leaked registrations. Existing shared-model tests now construct their
models in a single explicit domain.

`SingleWindowModel::from_entity` creates an independent window mount from an
existing model. This permits shared-domain application setup without allocating
a separate model runtime for the root. A test retains two adapters of one model,
checks distinct light/dark presentations, drops them independently and verifies
closed-model rejection. The constructor using a plain value uses this same path.

All 273 affected tests pass across 37 binaries, with all features and no skips;
runtime Clippy and structural checks pass. Child composition and application-wide
domain ownership still require migration; this does not complete application
isolation or the remaining plan lots.

### Explicit service publication

Added domain-local typed service resolution to `ModelRuntime`, `ModelContext`
and view `Context`. A caller-owned `ServiceRegistration` publishes a service;
it can be transferred to an application resource scope. Duplicate types are
rejected rather than silently replaced. The registry stores weak references to
avoid retaining a service/model/runtime cycle. Existing consumer references may
outlive publication, a distinction documented in `services.md`.

Three integration tests verify sharing across mounts, domain isolation, missing
services, duplicate rejection, scope removal and service-to-model retention.
All 115 runtime tests pass with all features; structural checks pass. Wiring the
application owner and demonstrating services in Shared Mailbox remain required;
the complete plan is not implemented by this registry alone.

### Shared Mailbox gallery page

The gallery now exposes **Examples → Shared Mailbox**. A render-independent
mailbox model holds 1,000 explicitly local messages. Two view models retain
independent selection and virtual scroll offsets while tracking that same mailbox.
Marking a message read/unread updates both lists and the shared unread counter.
The right view can be unmounted and remounted without destroying retained data.
Only visible rows are built, using the existing `VList` and `Button` widgets.

A public gallery interaction test verifies independent selection, shared read
state, virtual row bounds and updates across unmount/remount. All 31 gallery tests
pass with all features; structural checks pass. The existing layout sweep also
includes this navigation page. No visual browser/native validation has yet been
performed. These are two views in one window, not the required second native
window: that integration, local/model task demonstration and service usage remain
open, alongside the other incomplete plan lots.

### Shared Mailbox task lifetimes

The mailbox page now exposes real asynchronous subject-word counting at model
scope and separately at view scope. Both process the local dataset in yielding
batches; no network request or artificial latency is simulated. Starting again
cancels the previous handle. Visible status reports completion, launch failure
or cancellation when a view is unmounted and subsequently remounted.

A gallery test launches both operations, unmounts the right view before draining
completions, verifies the model result survives, then remounts and successfully
restarts the cancelled view operation. Another test verifies launch errors when
no executor is attached. All 33 gallery tests, targeted Clippy and structural
checks pass with all features. The second native window, services demonstration
and actual visual validation remain outstanding, as do the other plan lots.

### Named window adapter

`SingleWindowModel::window_key` binds the retained adapter to an explicit window
key instead of hardcoding `main`. View routing, task delivery and effect retrieval
all check that key. Frames and layout now reject foreign windows as well; they
previously invoked the component unconditionally. The regression verifies that
foreign layout/frames do not execute and foreign clipboard retrieval does not
consume the named window's pending request.

This prepares the auxiliary mailbox host but does not yet add its native window
to the gallery application. Runtime/DevTools/gallery validation and the remaining
plan requirements are tracked separately from this adapter change.

### Auxiliary mailbox application routing

The gallery launcher now uses `GalleryApplication`, retaining independent window
adapters. Shared Mailbox issues a real native `OpenWindow` command, with
`CloseWindow` behavior rather than quitting the application. Its auxiliary view
shares mailbox data but owns its selection/scroll and view task. Closing or
failing that window removes the adapter; reopening creates a new presentation.
The Web button is disabled and the application does not create an auxiliary Web
adapter. No in-page column is represented as a native window.

`SingleWindowModel` now explicitly closes its mount when dropped, even if another
component retains its event router. A headless application test checks the open
command, shared unread state, native-close routing and reopening with independent
selection. Runtime/gallery tests pass 150 cases across 13 binaries; targeted
Clippy and structural checks pass. The gallery wasm32 all-target build succeeds.
Actual native window rendering, browser interaction, and failure-path UX still
need validation; the complete plan remains unfinished.

### Auxiliary-window state and failure reporting

Shared Mailbox now displays closed/opening/ready/failed state from real application
window events. A failure includes the supplied error and invalidates the main
window so it is visible. Repeated clicks while opening do not issue duplicate
requests; clicking an already-ready window requests focus. Creation is gated by
the requested state, so a late view request after close/failure cannot resurrect
the auxiliary presentation. Web displays its actual in-page-only capability.

The application tests exercise failure after adapter creation, retry, ready/focus,
duplicate opening and late view requests after both failure and close. All 35
gallery tests, targeted Clippy and structural checks pass. This is headless event
protocol validation, not proof of native visual behavior or completion of the plan.

### Browser execution check

Rebuilt the gallery with `serve-widget-gallery.sh` and served it locally on port
8793. The connected browser has no usable WebGPU adapter, so it cannot validate
the rendered page. A separate Chromium launched with the repository's software
WebGPU test flags successfully runs the new `tests/pages/shared_mailbox.mjs`:
navigation, shared-model async completion without pointer input, right-view
unmount/remount and disabled native-window semantics all pass. Both page errors
and console errors are checked; none were reported in this run.

The software-browser screenshot is blank even after waiting for animation frames.
Consequently this run verifies browser execution and accessibility-driven actions,
not visual rendering, pointer hit testing or native window behavior. Screenshot
readback/compositing versus renderer behavior still needs diagnosis; do not treat
the successful interaction script as evidence of visual completion.

### Native command delivery correction

The retained event-router path collected `ContextEffects` in `Application` but
discarded their application commands. Direct `SingleWindowModel` tests did not
exercise that host boundary. Commands are now retained by the window runtime and
drained by `MultiApplication::process_pending`, including commands collected from
model/task wakeups, layout callbacks and animation callbacks. Execution remains
outside the model borrow and uses the existing bounded command loop.

Validation: 10 targeted runtime/gallery application tests passed; targeted
all-feature Clippy and wasm32 checks passed, as did Rust size and test-layout
checks. A real Wayland gallery was launched and closed through its native close
action. Its internal buttons were not exposed through the GTK accessibility tree,
and desktop screenshot access was denied, so opening the auxiliary window has
not been visually verified here. Custom elements and Timeline remain unimplemented;
this correction does not complete lots E–I or the remaining lifecycle requirements.

### Custom leaf pipeline and timeline consumer

The custom leaf extension now connects immutable typed properties, per-layout-node
scratch state, explicit layout/paint revisions, intrinsic measurement, geometry
preparation and renderer-neutral painting. Invalid measurements and duplicate custom
keys are errors. DevTools identifies the consumer type. A Custom Timeline gallery
page uses this API for its ruler, with ordinary widget clips and controls for drag,
selection, duration changes, keyboard editing and zoom.

See [the public contract and remaining requirements](custom-elements.md). This
is an implemented subset of E–I, not a claim of complete custom containers, custom
subregions, semantic subtrees or a shared second timeline presentation.

### Shared timeline presentations

The gallery now retains two independent `TimelineView` entities observing one
render-free clip entity. Clip movement and duration changes invalidate both views
and their standard detail lists. Selection, zoom and gesture origins remain local.
Drag cancellation restores position without overwriting a duration edit from the
other presentation. Targeted tests exercise cross-view edits, independent selection
and zoom, and cancellation after a concurrent duration change. Custom containers
and custom semantic/interactive regions are still the remaining implementation work.

### Layout algorithm extension boundary

Replaced the high-level `TaffyTree` storage with Argui-owned retained nodes and
Taffy's public low-level algorithms. Flexbox, grid, block layout, intrinsic leaf
measurement, layout caching and pixel rounding now use one algorithm dispatch
point. This is the prerequisite for custom containers recursively measuring and
placing ordinary children, not a second fallback layout implementation.

The initial migration passed all 76 layout tests. A root-type replacement leaked
detached graph nodes; reconciliation now removes that subtree explicitly.
`LayoutEngine::retained_node_count` and a passing 1,000-cycle test verify live graph
storage returns to the expected count after each root/subtree replacement. The
custom-container callback and region contract still need to be connected here.
