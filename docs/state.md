# Application state

`UiApp` is the small state boundary owned by `argui-runtime`:

```rust
trait UiApp {
    fn view(&self) -> Element;
    fn update(&mut self, event: &UiEvent) -> ViewUpdate;
}
```

`update` mutates plain application-owned Rust data. It returns
`ViewUpdate::Rebuild` only when `view` may have changed. The runtime lowers the
new `Element`, and `UiTree` classifies the actual difference as:

- `TreeUpdate::None`: the rebuilt description is identical;
- `TreeUpdate::Paint`: only quad or interaction visuals changed;
- `TreeUpdate::Layout`: structure, keys, text, layout, clipping, or hit-test
  geometry changed.

A paint update reuses Taffy geometry, shaped text, hit regions, and stable node
IDs. A layout update reconciles keyed identities before recomputing layout and
text. Returning `Rebuild` unnecessarily is therefore correct but measurable;
returning `None` avoids even rebuilding the view.

The runtime emits `RuntimeEvent::ViewUpdated(TreeUpdate)` for diagnostics. It
does not hide subscriptions, dependency tracking, global state, or an async
executor. A future DSL lowers into `Element` through the same `view` boundary.
