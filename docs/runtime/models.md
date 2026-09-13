# Models, presentations and ownership

`Entity<T>` retains shared model data and its model-owned resources. `T` does not
need to implement `Render`: services and background models use the same
`ModelRuntime`, transactions and task executor as presented models.
`Entity::update` supplies a `ModelContext`, which exposes model mutation,
subscriptions, services, commands and tasks without window focus or layout APIs.

A renderable entity can create independent `Mount<T>` presentations. Cloning a
mount retains that presentation; mounting the entity again creates a new
`MountId`. Each mount owns its environment, render cache, handlers, dependencies
and view tasks. `Context` provides the presentation capabilities during render
and view callbacks. `Context::mount_id()` identifies the owning presentation.

`Context::entity()` retains a child presentation and reuses its cached subtree
while it remains clean. Calling it twice with the same model creates two distinct
child presentations. Omitting a previously retained child on the next render
unmounts it. Model data survives while another strong entity or mount retains it.
`WeakEntity` can upgrade surviving model data; `WeakMount` cannot recreate a
presentation. `AnyEntity` is a **strong**, type-erased presentation handle.

## Visibility and lifetime

`Mount::set_visible(false)` hides a presentation without closing it.
`Context::entity_visible(&child, false)` retains a hidden child in the parent's
composition. Use the same call with `true` to show it again. A hidden presentation
returns an empty element without invoking `Render::render`, ignores UI handlers,
layout and animation frames, and retains its subscriptions and tasks. This is
not task suspension: model events and task completions continue to run.

The next visible render uses current model data and the current window
environment. Hidden children do not propagate presentation updates into their
parent's frame requests. Application commands remain deliverable. Explicit
`close()` ends a mount irreversibly, even if handles remain; showing it afterward
returns `ScopeClosed`. Closing the model's resource scope closes all its mounts.
Dropping the final strong model owner destroys its data.

The native host combines explicit visibility, occlusion and available minimized
state, then propagates effective visibility to presentations. Browser tab
visibility has no dedicated document-visibility adapter. Lifecycle notifications
are queued and delivered by the model dispatcher, as described below.

## Rendering

Model notification invalidates dependent presentations. When a dirty presentation
renders, `UiTree` classifies the actual difference as:

- `TreeUpdate::None`: the rebuilt description is identical;
- `TreeUpdate::Semantics`: only the accessibility description changed;
- `TreeUpdate::Paint`: paint data changed without requiring layout;
- `TreeUpdate::Scroll`: retained scroll geometry needs to be translated;
- `TreeUpdate::Layout`: structure, keys, text, layout, clipping, or hit-test
  geometry changed.

A paint update reuses Taffy geometry, shaped text, hit regions, and stable node
IDs. A layout update reconciles keyed identities before recomputing layout and
text. The runtime emits `RuntimeEvent::ViewUpdated(TreeUpdate)` for diagnostics.
The optional [tasks API](tasks.md) provides owned async work with UI-thread
completion; applications without that feature do not depend on Tokio.
A future DSL lowers into `Element` through the same `Render` boundary.

## Observations and typed events

Create cooperating models in the same `ModelRuntime`. `Context::read` records a
render dependency; rebuilding without that read detaches it. `observe` returns
a subscription to retain explicitly. Model-owned observation invalidates the
dependent model; presentation-owned observation invalidates that mount's cache.
Tracked reads and subscriptions require the same model domain.

An `EventEmitter<E>` publishes typed events through `emit`; `subscribe` supplies
the recipient callback. Keep the returned `Subscription` alive. Dropping it or
closing either endpoint scope suppresses queued delivery as well. Callbacks run
after model borrows end. Use weak handles in owner-held callbacks to avoid cycles.

Invalidations coalesce within transactions and terminate dependency cycles.
Events, invalidations and lifecycle delivery share a 64-step dispatch budget.
Typed events have a capacity of 1,024 including the event being delivered;
overflow returns an explicit error. Hosts wake for pending work without polling.
A hostless consumer must drain `ModelRuntime::dispatch_pending` at safe points.

## Runtime ownership

The lifetime of each resource follows its explicit owner:

| Resource | Creator | Strong owner | End of ownership | Allowed survivors |
| --- | --- | --- | --- | --- |
| Native application runtime | `run_app` / multi-application launch | Event-loop application handler | Handler destruction / application exit | Externally retained models and service values; no live window presentation |
| Window runtime | Multi-application window opening, or single-window launch | `Application` / multi-application window entry | Window removal or application destruction | Shared model, other window entries, application-owned executor |
| Presentation | `Entity::mount`, retained child composition | `Mount`, parent presentation, or type-erased host router | Explicit close, parent/model scope closure, or final presentation handle drop | Model data and independent mounts; retained closed handles cannot render or handle UI events |
| Model data | `Entity::new`, `ModelRuntime::entity`, context entity creation | Strong entity and mount handles | Last strong model owner drops | No model data; weak handles cannot recreate destroyed data |
| Model resources | Model construction | Model's `ResourceScope` | Explicit model scope close or model destruction | Data can remain readable through strong handles after explicit scope closure |
| Service publication | `ModelRuntime::register_service` | Returned `ServiceRegistration`, optionally owned by a scope lease | Registration or owning scope is dropped/closed | Existing `Rc` consumers retain the service value; lookup no longer finds the publication |
| View task | Mount or presentation context | Executor registration, cancelled by both view and model scopes | Completion, explicit cancellation, mount/model closure, executor shutdown | Task handle and cancellation token; cancelled result cannot invoke the callback |
| Model task | Model context | Executor registration, cancelled by model scope | Completion, cancellation, model closure, executor shutdown | Closing one window alone does not cancel it |
| Model event subscription | Model context | Subscription plus endpoint scope registrations | Subscription drop or endpoint closure | Other model subscriptions and surviving endpoints |
| View event subscription | Presentation context | Subscription plus mount and endpoint scope registrations | Subscription drop, mount closure, or endpoint closure | Model-owned subscriptions and independent mount subscriptions |

Hiding a view preserves its ownership. It suppresses presentation work and UI
input, without cancelling model events or tasks.

`SingleWindowModel` closes its mount on drop. `Application` closes its type-erased
presentation on drop; a standalone application's executor is shut down there.
`MultiApplication` shuts down the shared executor on drop, and dropping window
entries closes their presentations. These are source-level ownership paths, not
proof that every OS error or exit path reaches them correctly.

### Retained models after application exit

Model data is not implicitly destroyed by closing its last window if an external
strong handle remains. It can still be read and mutated. Model subscriptions and
service registrations that external code still owns are not application-owned
merely because they were used by a window. Their explicit owner must close them
if their lifetime should end with the application.

An executor retained after `TaskRuntime::shutdown` rejects new task registration
with `TaskError::Unavailable` and discards queued task callbacks. A model retaining
that executor therefore cannot schedule new work on it. A closed mount cannot be
reopened; a live model may create a new mount, whose attachment to a new host is
an explicit operation. An old host's wake callback does not constitute a new
presentation or extend the old window's lifetime.

### Reentrant presentation closure

A child frame may close its parent. Frame traversal releases its child-list
borrow before invoking callbacks, and checks closure before delivering a parent
frame. Closed siblings are skipped. If rendering closes its own mount, the
returned subtree and newly declared handlers are discarded instead of being
installed into the closed presentation. The retained data remains readable.

`ResourceScope` runs all cleanup callbacks in reverse registration order even
when one panics, then resumes the first panic when not already unwinding.
Lifecycle ordering and observer panic behavior are described below. Platform
error-path validation remains part of the [roadmap](../roadmap.md).

## Presentation lifecycle notifications

`ModelRuntime::observe_mounts` returns a cancellable `Subscription` for future
`MountEvent` records in that model domain. Records contain the `EntityId`,
`MountId` and one of `Mounted`, `VisibilityChanged { visible }`, or `Unmounted`.
They retain neither the model nor its presentation. Subscribe before mounting
when the initial transition matters; subscription does not replay old records.

A successful `Entity::mount` records `Mounted` once. Direct retained rendering
through `Entity::render` starts its presentation on its first render instead.
Repeated renders do not remount. Visibility records are emitted only when the
visibility value changes. Explicit close followed by drop produces one unmount.
A first render that panics still balances an already created mount with an
unmount when that mount is dropped.

Mount creation is ordered parent before children. Closing or dropping a composed
parent records children before parent. Sibling cleanup follows resource-scope
reverse registration order; consumers should identify presentations by ID rather
than infer their identity from sibling position.

### Delivery and reentrance

Records are queued. `ModelRuntime::dispatch_pending` delivers them outside active
transactions; normal transaction exit never invokes lifecycle observers. Hosts
call this dispatcher on the model wake, outside render, layout and paint. A
hostless consumer must call it at the equivalent safe point. In particular,
rendering a child from another model domain cannot invoke its lifecycle observer
inside the parent's render borrow.

Lifecycle, typed events and invalidations share the dispatcher's 64-step work
budget. Pending lifecycle records request another host wake. An idle dispatcher
requests no polling timer. `pending_mount_events` reports undelivered records,
including records with remaining observers after partial delivery. The budget
bounds work per turn, not the number of records a producer can enqueue before
the host drains them.

Observers may mutate models, close mounts or request a new mount. Those changes
append transitions for later delivery in the same or subsequent bounded batch.
Cancelling or dropping the subscription prevents its queued callbacks as well.
A callback added after a transition was recorded does not receive that record.
Use weak model handles in callbacks when the model owns the subscription, to
avoid an application-created reference cycle.

### Panic and destruction policy

If an observer panics, the panic propagates from `dispatch_pending`. Its position
has already advanced: the panicking delivery is not retried, and other observers
remain queued for a subsequent dispatch if the host recovers. Presentation cleanup
has completed before an unmount observer runs, so an observer panic cannot prevent
that cleanup. `ResourceScope` continues other cleanup callbacks before propagating
its first panic.

Unmount does not destroy shared model data. Data lives until the final strong
model or mount owner is dropped. There is no model-destruction callback that could
resurrect that value; lifecycle records contain only historical IDs. The resource
and task policy is detailed in [ownership](#runtime-ownership).

Behavioral coverage lives in the runtime's
[model tests](../../crates/argui-runtime/tests/model/), including lifecycle,
visibility, scope cleanup and host dispatch.

## Shared services

A `ModelRuntime` defines an explicit domain. `register_service(value)` publishes
one service per type and returns a `ServiceRegistration<T>` to retain. Duplicate
registration fails without replacing the original. Independent runtimes never
resolve each other's services.

The application owns registrations directly or through `ResourceScope::own`.
In the latter case, retain the returned `ResourceLease` as well. Closing that
scope removes its services; closing a view does not remove application-owned
services.

`ModelRuntime::service::<T>()`, `ModelContext::service::<T>()` and
`Context::service::<T>()` return `Option<Rc<T>>` without retaining a registry
borrow. A detached context or missing service returns `None`; there is no
global lookup or implicit creation.

Removing a registration prevents new lookups, but existing `Rc<T>` consumers
can finish their work. A service that must stop existing operations needs an
explicit shutdown contract. The registry stores weak references, so a service
containing models does not itself create a registry/service/model/runtime cycle.

Services complement observable models. A mailbox can register a synchronization
service while messages remain in `Entity` values observed by its views. Resolving
a service creates no render dependency; notifications still flow through models.

The [service tests](../../crates/argui-runtime/tests/model/services.rs) cover
shared identity, model/view contexts, isolated domains, duplicates, scoped
removal and retention cycles.
