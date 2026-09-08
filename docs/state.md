# Application state

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

These presentation APIs do not yet define the integration of native window
minimization or browser tab visibility. The [lifecycle notifications](lifecycle.md) are delivered by the model dispatcher.
Their host integration and validation are tracked in [Point 1 completion](point-1-completion.md).

## Rendering

Model notification invalidates dependent presentations. When a dirty presentation
renders, `UiTree` classifies the actual difference as:

- `TreeUpdate::None`: the rebuilt description is identical;
- `TreeUpdate::Paint`: only quad or interaction visuals changed;
- `TreeUpdate::Layout`: structure, keys, text, layout, clipping, or hit-test
  geometry changed.

A paint update reuses Taffy geometry, shaped text, hit regions, and stable node
IDs. A layout update reconciles keyed identities before recomputing layout and
text. The runtime emits `RuntimeEvent::ViewUpdated(TreeUpdate)` for diagnostics.
The optional [tasks API](tasks.md) provides owned async work with UI-thread
completion; applications without that feature do not depend on Tokio.
A future DSL lowers into `Element` through the same `Render` boundary.
