# Runtime ownership

This matrix describes the current ownership boundaries. Host integration evidence
and the remaining acceptance checks are tracked in [Point 1](point-1-completion.md).

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
input, without cancelling model events or tasks. Native window visibility and
minimization still require the host integration work identified in Point 1.

`SingleWindowModel` closes its mount on drop. `Application` closes its type-erased
presentation on drop; a standalone application's executor is shut down there.
`MultiApplication` shuts down the shared executor on drop, and dropping window
entries closes their presentations. These are source-level ownership paths, not
proof that every OS error or exit path reaches them correctly.

## Retained models after application exit

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

## Reentrant presentation closure

A child frame may close its parent. Frame traversal releases its child-list
borrow before invoking callbacks, and checks closure before delivering a parent
frame. Closed siblings are skipped. If rendering closes its own mount, the
returned subtree and newly declared handlers are discarded instead of being
installed into the closed presentation. The retained data remains readable.

`ResourceScope` runs all cleanup callbacks in reverse registration order even
when one panics, then resumes the first panic when not already unwinding.
This scope guarantee does not yet define lifecycle-notification ordering or prove
application-wide cleanup under panic; those remain part of P1-02/P1-03.
