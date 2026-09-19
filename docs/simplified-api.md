# The 0.3 application API

Argui 0.3 keeps the retained event engine, but ordinary application code no
longer needs to look like an event router. The common path is a local widget
handler with a typed payload. State remains controlled by the model.

## 1. Install and open a window

The `basic` profile enables tasks and the common form, selection, and overlay
widgets. Granular `widget-*` features remain available for smaller builds.

```toml
[dependencies]
argui = { version = "0.3.1", features = ["basic"] }
```

Application code can use the small prelude and keep platform configuration
explicit:

```rust,ignore
use argui::prelude::*;

#[derive(Default)]
struct Hello;

impl Render for Hello {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("Hello from Argui")
    }
}
```

See [application setup](platform/application.md) for identity, renderer, native,
WebAssembly, Android, and iOS entry points.

## 2. A local callback

`Context::callback` receives mutable model state and automatically invalidates
the current presentation after it returns:

```rust,ignore
Button::new("increment", "Increment", theme.button())
    .on_click(cx.callback(|counter| {
        counter.count += 1;
    }))
    .build()
```

The handler is additive. Pointer, touch, Enter, Space, and accessibility click
activation all use the widget's existing event pipeline. A direct handler only
fires for that widget's own target, not because a descendant event bubbled
through it.

Use `Context::event_handler` when the callback needs the event or context:

```rust,ignore
Button::new("save", "Save", theme.button())
    .on_click(cx.event_handler(|editor, event, cx| {
        editor.save();
        event.stop_propagation();
        cx.notify();
    }))
    .build()
```

This form does **not** invalidate implicitly.

## 3. A controlled form

Widgets emit domain values; the application stores them and passes them back on
the next render:

```rust,ignore
let email = cx.input_callback(|form, value| form.email = value);
let accepted = cx.value_callback(|form, value| form.accepted = value);
let submit = cx.submit_callback(|form, _value| form.submissions += 1);

Element::column([
    Input::new("email", &self.email, "Email", theme.input())
        .label("Email")
        .on_input(email)
        .on_submit(submit)
        .build(),
    Checkbox::new("terms", "Accept terms", self.accepted)
        .on_change(accepted)
        .build(&theme),
])
```

The same rule applies to `on_select`, `on_change`, `on_commit`,
`on_open_change`, `on_action`, and `on_activate`. Disabled, busy, constrained,
or unavailable choices do not emit direct values.

## 4. Test the form without a window

`argui-testing` uses the real retained tree, layout engine, hit regions, focus,
editing, and event dispatch without Winit, a display server, or a GPU:

```rust,ignore
use argui::accessibility::Role;
use argui_testing::TestApp;

let mut app = TestApp::new(Form::default());
app.get_by_role(Role::TextInput, "Email")
    .type_text("person@example.com")?;
app.get_by_role(Role::CheckBox, "Accept terms").click()?;
app.get_by_role(Role::TextInput, "Email").submit()?;

app.assert_input_value("email", "person@example.com");
app.assert_focused("email");
app.assert_quiescent();
# Ok::<(), argui_testing::TestError>(())
```

Queries by key, role/name, text, label, semantic state, and focus fail on zero or
multiple matches with the focused node, close candidates, and a compact semantic
tree. The harness also supports pointer movement, touch, keyboard shortcuts,
Tab traversal, replacement, paste, wheel input, drag, accessibility actions,
resize, environment changes, lifecycle events, clipboard and effect inspection.

`TestWindows` presents one entity in multiple independent windows. Each window
has its own focus, layout, handler slots, and presentation resources while model
state remains shared.

## 5. Tasks and loading state

Keep the returned handle while work is wanted. `spawn_latest` cancels a previous
operation in the same slot and prevents stale delivery:

```rust,ignore
cx.spawn_latest(&mut self.search, load(query), |model, result, cx| {
    model.loading = false;
    model.results = result.unwrap_or_default();
    cx.notify();
})?;
```

Native headless tests use a paused executor. `app.advance(duration)` advances
animations, timers, and `tasks::sleep` without wall-clock sleeping;
`run_until_idle` drains ready work. Closing a presentation or model cancels its
owned task before a queued result can reach a stale callback.

## 6. Reusable child models

Use `Entity<T>` when a child owns state, lifecycle, subscriptions, or tasks.
Render it with `cx.entity(&child)`. The parent can still use plain view functions
for stateless sections. Multiple mounts retain independent presentations over
the same model identity.

## 7. Typed actions and shortcuts

Behavior action enums remain the runtime-neutral layer for reducers, custom
controls, and centralized command systems. `Context::action_handler` binds a
typed application action. Application shortcuts still belong on a deliberate
ancestor listener so one handler can observe the intended scope.

## 8. Delegation, capture, and bubbling

`Context::listener` and `Element::on` are not deprecated. Use them when one
ancestor intentionally routes many descendants, or when capture, passive,
one-shot, propagation, or default-action control is the feature being built:

```rust,ignore
Element::column(children).on(cx.listener(EventType::Click, |app, event, cx| {
    match event.target_key() {
        Some("save") => app.save(cx),
        Some("delete") => app.delete(cx),
        _ => {}
    }
}))
```

An ordinary button does not require this pattern. See
[interaction](ui/interaction.md) for phase and cancellation rules.

## 9. Custom controls

Compose `Interaction`, semantics, gestures, and listeners directly when a
control is not represented by a widget. Keep the renderer/runtime independent
from application DSLs, provide keyboard and accessibility equivalents, and test
pointer activation with `TestApp::click` so real hit testing remains covered.
The [custom-element guide](ui/custom-elements.md) covers custom layout and paint.

The complete per-widget handler and payload inventory is in
[widget interaction APIs](widgets/interaction-api.md). Migration examples are in
[the 0.3 guide](migrations/0.3.md).
