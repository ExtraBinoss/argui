# Models, presentations, and ownership

`Entity<T>` owns shared model data. `T` does not need to render: services and
background models use the same runtime, transactions, events, and tasks as UI
models.

A renderable entity can create several `Mount<T>` presentations. Each mount has
its own environment, render cache, handlers, dependencies, and view-owned
resources.

| Handle | Owns | Can be shared |
| --- | --- | --- |
| `Entity<T>` | model data and model resource scope | yes |
| `WeakEntity<T>` | no ownership | upgrades while model data exists |
| `Mount<T>` | one presentation of an entity | yes |
| `WeakMount<T>` | no ownership | upgrades while that mount exists |
| `AnyEntity` | type-erased strong presentation | yes |
| `Context<T>` | temporary presentation capabilities | only during callbacks |
| `ModelContext<T>` | temporary model capabilities | only during mutation |

Mounting one entity twice creates two independent presentations. Calling
`Context::entity` retains a child mount across parent rebuilds; omitting it on
the next render unmounts that child.

## Rendering and invalidation

`Render::render` returns the current `Element` description. Reads recorded
through the context create dependencies. `notify()` invalidates presentations
that depend on the changed model; repeated invalidations in one transaction
coalesce.

After reconciliation, `UiTree` classifies the smallest required update:

| Update | Work |
| --- | --- |
| `None` | reuse the complete presentation |
| `Semantics` | update accessibility only |
| `Paint` | reuse layout, shaped text, hit regions, and node IDs |
| `Scroll` | translate retained scroll and hit geometry |
| `Layout` | recompute affected layout, text, and paint |

Keyed siblings preserve stable node identity when reordered. Each mount caches
its own rendered subtree; sharing model data does not share window environment
or presentation handlers.

## Visibility and closure

`Mount::set_visible(false)` keeps a presentation alive but suppresses render,
layout, input, and animation frames. Its subscriptions and tasks continue.
Showing it again renders current model data with the current environment.

`close()` is irreversible. Closing a model resource scope closes its mounts and
cancels model-owned resources. Dropping the last strong model owner destroys the
data. Closing one window does not destroy a model retained elsewhere.

The native host combines requested visibility with occlusion and minimized state.
Hidden presentations do not propagate frame requests into their parent.

## Observing models

Create cooperating entities in the same `ModelRuntime`.

- `Context::read` records a render dependency for the current mount.
- `observe` returns an explicit `Subscription`.
- `EventEmitter<E>` and `subscribe` carry typed events between models.
- `emit` queues delivery after active borrows end.

Keep each subscription alive for as long as delivery is wanted. Dropping it or
closing either endpoint suppresses queued callbacks. Use weak handles in a
callback retained by its owner to avoid reference cycles.

Delivery is bounded per dispatcher turn. Hosts wake again when work remains;
they do not poll while idle. A consumer using `ModelRuntime` without an Argui
host must call `dispatch_pending` at a safe point outside model borrows.

## Tasks

Model tasks belong to the model scope. View tasks additionally belong to one
mount, so unmounting cancels them. Keep the returned `TaskHandle` or
`TaskSlot`; dropping it cancels delivery.

`spawn_latest` replaces older work and rejects results already queued by the
replaced task. Executor shutdown rejects new work and suppresses queued
callbacks. The full API is in [owned asynchronous tasks](tasks.md).

## Services

A `ModelRuntime` is an explicit service domain.
`register_service(value)` publishes one value per Rust type and returns a
`ServiceRegistration<T>`; retain that registration while lookups should work.
Duplicate registration fails without replacing the first value.

`ModelRuntime::service`, `ModelContext::service`, and `Context::service`
return `Option<Rc<T>>`. Removing a registration blocks new lookups while
existing `Rc<T>` users can finish. Service lookup does not create a render
dependency; observable application state still belongs in entities.

## Resource lifetime

| Resource | Ends when |
| --- | --- |
| Presentation | mount closes, parent/model scope closes, or final strong mount drops |
| Model data | final strong entity or mount owner drops |
| Model task | completes, is cancelled, model closes, or executor stops |
| View task | model task conditions plus mount closure |
| Subscription | handle drops or either endpoint closes |
| Service publication | registration drops or its owning scope closes |
| Window runtime | window entry is removed |
| Application executor | application host shuts down |

`ResourceScope` owns cleanup callbacks and runs them in reverse registration
order. Hiding is not closure and does not change ownership.

## Lifecycle events

`ModelRuntime::observe_mounts` reports future `Mounted`,
`VisibilityChanged`, and `Unmounted` records with `EntityId` and `MountId`.
It does not replay old records or retain the model. Parent mounting precedes
child mounting; child unmounting precedes parent unmounting.

Lifecycle callbacks run through the same bounded dispatcher as model events and
invalidations, outside render, layout, and paint. A callback may mutate models
or close mounts; resulting records are queued for later delivery.

Behavioral coverage lives in
[runtime model tests](../../crates/argui-runtime/tests/model/). Public method
details and failure conditions live in the crate Rustdoc.
