use argui_runtime::{Context, Entity, Render};
use argui_ui::Element;

#[test]
fn data_revision_tracks_notifications_without_a_render_implementation() {
    let model = Entity::new(0);
    assert_eq!(model.revision(), 0);
    model.update(|value, _| *value += 1);
    assert_eq!(model.revision(), 0);
    model.runtime().transaction(|| {
        for _ in 0..100 {
            model.update(|value, cx| {
                *value += 1;
                cx.notify();
            });
        }
    });
    assert_eq!(model.revision(), 100);
    assert_eq!(model.read(|value| *value), 101);
    assert_eq!(model.clone().revision(), 100);
}

struct View;
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text("revision")
    }
}

#[test]
fn rendering_does_not_reset_or_advance_the_model_revision() {
    let view = Entity::new(View);
    let mount = view.mount().unwrap();
    let _ = mount.render(Default::default()).unwrap();
    assert_eq!(view.revision(), 0);
    view.update(|_, cx| cx.notify());
    assert_eq!(view.revision(), 1);
    let first = mount.render(Default::default()).unwrap();
    let cached = mount.render(Default::default()).unwrap();
    assert_eq!(first, cached);
    assert_eq!(view.revision(), 1);
    mount.update(|_, cx| cx.request_paint()).unwrap();
    assert_eq!(view.revision(), 1);
}
