# Presentation lifecycle notifications

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

## Delivery and reentrance

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

## Panic and destruction policy

If an observer panics, the panic propagates from `dispatch_pending`. Its position
has already advanced: the panicking delivery is not retried, and other observers
remain queued for a subsequent dispatch if the host recovers. Presentation cleanup
has completed before an unmount observer runs, so an observer panic cannot prevent
that cleanup. `ResourceScope` continues other cleanup callbacks before propagating
its first panic.

Unmount does not destroy shared model data. Data lives until the final strong
model or mount owner is dropped. There is no model-destruction callback that could
resurrect that value; lifecycle records contain only historical IDs. The resource
and task policy is detailed in [ownership](ownership.md).

Shared Mailbox displays counters from delivered lifecycle records. Host shutdown
delivery and visibility propagation from native/browser windows remain subject to the remaining
[Point 1 acceptance checks](point-1-completion.md).
