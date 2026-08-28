# Application state

`Render` is the retained component boundary owned by `argui-runtime`:

```rust
trait Render {
    fn render(&mut self, cx: &mut Context<Self>) -> Element;
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>);
}
```

`Entity<T>` owns component-local state and caches the exact `Element` subtree
returned by `T`. `Context::notify()` marks only that entity dirty;
`Context::entity()` returns a child's cached COW subtree while it remains clean.
`WeakEntity<T>` and `AnyEntity` provide non-owning and type-erased identities.

When a dirty entity renders, `UiTree` classifies the actual difference as:

- `TreeUpdate::None`: the rebuilt description is identical;
- `TreeUpdate::Paint`: only quad or interaction visuals changed;
- `TreeUpdate::Layout`: structure, keys, text, layout, clipping, or hit-test
  geometry changed.

A paint update reuses Taffy geometry, shaped text, hit regions, and stable node
IDs. A layout update reconciles keyed identities before recomputing layout and
text. Returning `Rebuild` unnecessarily is therefore correct but measurable;
clean entities avoid rebuilding their view entirely.

The runtime emits `RuntimeEvent::ViewUpdated(TreeUpdate)` for diagnostics. It
does not hide subscriptions, dependency tracking, global state, or an async
executor. A future DSL lowers into `Element` through the same `render` boundary.

`Render` is the only single-window component API. The showcase, DevTools and
`run_app` all use the same retained path; there is no immediate-mode adapter.
