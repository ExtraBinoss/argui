# Owned asynchronous tasks

Enable the `tasks` feature on `argui`. Tokio is the native backend; Web uses
browser-local futures and does not bundle Tokio. One lazy native executor is
shared by the application's windows, with at most four worker threads.

`ModelContext::spawn(future, callback)` returns `Result<TaskHandle, TaskError>`.
Keep the handle while the operation is wanted. Dropping it, cancelling it,
destroying its shared model, or closing its entity resource scope cancels delivery.
Hiding a retained component is not destruction. Closing one presentation does not
cancel shared-model work retained by another presentation.

`Context::spawn` additionally binds work to the calling presentation. Its callback
can use that presentation's services; unmounting cancels even queued delivery.
Use `spawn_in(scope, future, callback)` on either context for an additional lifetime
constraint, not a transfer of ownership. Mount destruction closes its scope; model
destruction cancels all of its remaining tasks. Runtime shutdown cancels delivery
regardless of surviving model or mount handles. There is no subtree-wide task
cancellation API: ownership follows explicit resource scopes.

For presentation-owned operations, `Mount::spawn(future, callback)` applies both
model and mount lifetimes automatically. `Mount::spawn_latest` replaces a
`TaskSlot` with the same cancellation rules. Completion uses the exact mount's
context and environment; a dead mount is never recreated through its model.
Both APIs reject a closed scope with `TaskError::ScopeClosed`, and require an
attached executor just like model tasks. Retain their handle or slot while the
operation is wanted.

`spawn_latest(&mut slot, future, callback)` on either context replaces a retained
`TaskSlot`. Even an older result already queued for delivery is discarded.
The callback receives the component, the typed result and the same kind of
context that launched the task (`ModelContext` or presentation `Context`). Only
the callback runs on the UI thread; native futures/results must be Send.
Call `cx.notify()` when a result changes visible state.

Start tasks from handlers or updates after the Entity is attached to its host,
not on every render. A detached Entity or plain default Context returns
`TaskError::Unavailable`. Custom hosts attach a `TaskRuntime` using
`Entity::set_task_runtime`, call `drain()` when its wake callback fires, and
use `AppModel::tasks_ready` to collect application invalidations and commands.
Call `shutdown()` when the host ends.

Errors returned by the future remain ordinary typed results inside the task
result. Native unwinding panics become `TaskError::Panicked`; aborting panics
(including browser traps) cannot be recovered.

The dispatcher accepts at most 256 outstanding tasks and delivers at most 64
completions per wake. It uses a bounded completion channel and coalesced wakes.
Spawning while full returns `CapacityExceeded`. No polling or permanent
animation timer runs. `tasks::sleep` and `tasks::yield_now` wake active work.

Native `spawn_blocking` passes a cooperative `CancellationToken`. A started
blocking function cannot be forcibly interrupted: check the token and ensure
the operation terminates. Cancellation suppresses its callback even if the
function finishes later. Browser code uses async operations or cooperative
chunks; this API does not create Web Workers.

Open **Examples → Async tasks** in the Widget Gallery. It searches 10,000
generated example draft titles, supports `id:500`, and reports a real parsing
error for `id:abc`. The 200 ms delay is debounce, not simulated I/O.
