# Hot reload

Argui can apply native Rust code patches to a running debug application through
[Subsecond](https://docs.rs/subsecond/0.7.10/subsecond/). The integration is an
optional Cargo feature and is absent from default builds.

```toml
[dependencies]
argui = { version = "0.3.2", features = ["hot-reload"] }
```

Install the matching Dioxus CLI and start the executable with hot patching:

```sh
cargo install dioxus-cli --version 0.7.10 --locked
dx serve --package my-app --features hot-reload --hot-patch
```

`dx serve` watches the crate, compiles changed functions and sends the patch to
the process. Argui reconnects that patch to its retained runtime: it invalidates
every open window, rebuilds the root view, and refreshes cached child entities.
Existing model and widget instances remain alive, so input values, counters,
selection, scroll state and open overlays can survive a patch.

The bridge covers every `Render` hook and the complete `AppModel` boundary. That
includes view construction, UI event handlers captured by a view, tasks,
animation frames, layout notifications, application events, assets, inspector
data and multi-window models. `RuntimeEvent::HotReloaded { generation }` is sent
after Argui schedules the refreshed views, which lets an application report or
measure successful patches.

## Keep editable code in the executable crate

Subsecond patches the final executable crate, sometimes called the tip crate.
Put application modules below `src/main.rs` so the patcher can replace them:

```rust
mod app;
mod pages;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::launch()
}
```

Calling all application code through a separate library crate limits what
Subsecond can replace. Shared framework and business crates can remain
libraries; move the render functions you want to edit into the executable
crate, or include those source modules from `main.rs`. The Widget Gallery uses
this layout and includes an interactive **Hot reload** page.

Run it from the repository root:

```sh
dx serve --package argui-widget-gallery --features hot-reload --hot-patch
```

Open **Examples → Hot reload**, increment the counter, then edit the
`live_copy` string in
`crates/argui-widget-gallery/src/pages/hot_reload.rs`. The copy changes while
the counter keeps its current value. The terminal also prints the applied
patch generation.

## Boundaries

- Hot patching currently runs on native targets and requires a debug build.
- Changing function bodies, view composition, styles and most event behavior
  is the intended loop.
- Changing a live type's memory layout, fields, trait signatures, Cargo
  features or dependencies requires a full restart.
- WebAssembly builds accept the feature as a no-op and keep the normal browser
  rebuild/reload loop.
- Release builds compile out Argui's connection and dispatch path. Enabling the
  feature can still add development dependencies to Cargo's build graph.

These limits follow Subsecond's patch model. Dioxus documents the same restart
boundary for changes such as struct fields in its
[hot-reload guide](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload/).
