Model mutations cannot request window focus, including for renderable models:

```compile_fail,E0599
use argui_runtime::{Context, Entity, Render};
use argui_ui::Element;
struct View;
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> Element { Element::text("view") }
}
Entity::new(View).update(|_, cx| cx.request_focus("input"));
```

They cannot inspect window layout:

```compile_fail,E0599
use argui_runtime::{Context, Entity, LayoutSnapshot, Render};
use argui_ui::Element;
struct View;
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> Element { Element::text("view") }
}
Entity::new(View).update(|_, cx| { cx.observe_bounds(&LayoutSnapshot::default(), "input"); });
```

Nor can they read a presentation's environment:

```compile_fail,E0599
use argui_runtime::{Context, Entity, Render};
use argui_ui::Element;
struct View;
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> Element { Element::text("view") }
}
Entity::new(View).update(|_, cx| { cx.environment(); });
```

These capabilities are available through a live presentation, while model
notification remains available without rendering:

```rust
use argui_runtime::{Context, Entity, LayoutSnapshot, Render};
use argui_ui::Element;
struct View;
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> Element { Element::text("view") }
}
let data = Entity::new(0_u32);
data.update(|value, cx| { *value += 1; cx.notify(); });
let model = Entity::new(View);
let mount = model.mount().unwrap();
mount.update(|_, cx| {
    cx.request_focus("input");
    let _ = cx.observe_bounds(&LayoutSnapshot::default(), "input");
    let _ = cx.environment();
}).unwrap();
```
