//! Deterministic retained-tree stress labs, shared by native and WebAssembly.

use argui::{
    paint::{Border, Color, CornerRadii},
    runtime::{Context, Entity, Render},
    text::{TextColor, TextStyle, TextWrap},
    ui::{
        Axes, Element, Overflow, ScrollConfig, Sides, UiEvent, UiEventKind, VariableList,
        VirtualList, percent, sides,
    },
};

const ROWS: usize = 1_000_000;

pub struct PerfShowcase {
    counter: Entity<CounterLab>,
    fixed: Entity<FixedListLab>,
    variable: Entity<VariableListLab>,
    renders: u64,
}

impl Default for PerfShowcase {
    fn default() -> Self {
        Self {
            counter: Entity::new(CounterLab::default()),
            fixed: Entity::new(FixedListLab::default()),
            variable: Entity::new(VariableListLab::default()),
            renders: 0,
        }
    }
}

impl Render for PerfShowcase {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        let counter_renders = self.counter.read(|lab| lab.renders);
        let fixed_renders = self.fixed.read(|lab| lab.renders);
        let variable_renders = self.variable.read(|lab| lab.renders);
        Element::column([
            Element::text("Argui retained performance laboratory").text_style(text(
                30.0,
                TextColor::WHITE,
                750,
            )),
            Element::text(format!(
                "root renders={} · counter={} · fixed-list={} · variable-list={}",
                self.renders, counter_renders, fixed_renders, variable_renders
            ))
            .keyed("perf-hud")
            .text_style(text(15.0, TextColor::rgb(0.48, 0.88, 0.68), 650)),
            cx.entity(&self.counter),
            cx.entity(&self.fixed),
            cx.entity(&self.variable),
        ])
        .padding(Sides::length(24.0))
        .gap(18.0)
        .width(percent(1.0))
        .height(percent(1.0))
        .background(Color::rgb(0.035, 0.045, 0.065))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default())
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if event.kind == UiEventKind::Clicked && event.key.as_deref() == Some("perf-counter") {
            self.counter.update(|lab, child| {
                lab.value += 1;
                child.notify();
            });
            cx.notify();
            return;
        }
        if let UiEventKind::Scrolled { offset, .. } = event.kind {
            match event.key.as_deref() {
                Some("perf-million-fixed") => {
                    let changed = self.fixed.read(|lab| {
                        lab.list.window(lab.offset).range != lab.list.window(offset.y).range
                    });
                    self.fixed.update(|lab, child| {
                        lab.offset = offset.y;
                        if changed {
                            child.notify();
                        }
                    });
                    if changed {
                        cx.notify();
                    }
                }
                Some("perf-million-variable") => {
                    let changed = self.variable.read(|lab| {
                        lab.list.window(lab.offset).range != lab.list.window(offset.y).range
                    });
                    self.variable.update(|lab, child| {
                        lab.offset = offset.y;
                        if changed {
                            child.notify();
                        }
                    });
                    if changed {
                        cx.notify();
                    }
                }
                _ => {}
            }
        }
        if event.kind == UiEventKind::Clicked
            && event.key.as_deref() == Some("perf-measure-variable")
        {
            self.variable.update(|lab, child| {
                let index = lab.list.window(lab.offset).range.start.saturating_add(2);
                let extent = 28.0 + (index % 7) as f32 * 9.0;
                let update = lab.list.measure(index, extent, lab.offset);
                lab.offset = update.corrected_offset;
                child.notify();
            });
            cx.notify();
        }
    }
}

#[derive(Default)]
struct CounterLab {
    value: u64,
    renders: u64,
}

impl Render for CounterLab {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        panel([
            Element::text("Local retained state").text_style(text(19.0, TextColor::WHITE, 700)),
            Element::text(format!("value={} (only this entity changes)", self.value))
                .keyed("perf-counter")
                .padding(sides(14.0, 9.0))
                .background(Color::rgb(0.12, 0.36, 0.28))
                .radius(CornerRadii::all(9.0)),
        ])
    }
}

struct FixedListLab {
    list: VirtualList,
    offset: f32,
    renders: u64,
}

impl Default for FixedListLab {
    fn default() -> Self {
        Self {
            list: VirtualList::new(ROWS, 28.0, 240.0).overscan(4),
            offset: 0.0,
            renders: 0,
        }
    }
}

impl Render for FixedListLab {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        let list = self
            .list
            .clone()
            .build("perf-million-fixed", self.offset, row);
        panel([
            Element::text("1,000,000 fixed-height rows").text_style(text(
                19.0,
                TextColor::WHITE,
                700,
            )),
            list,
        ])
    }
}

struct VariableListLab {
    list: VariableList,
    offset: f32,
    renders: u64,
}

impl Default for VariableListLab {
    fn default() -> Self {
        Self {
            list: VariableList::new(ROWS, 34.0, 260.0).overscan(5),
            offset: 0.0,
            renders: 0,
        }
    }
}

impl Render for VariableListLab {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        panel([
            Element::text("1,000,000 variable-height rows · anchored measurements")
                .text_style(text(19.0, TextColor::WHITE, 700)),
            Element::text("Measure a visible row")
                .keyed("perf-measure-variable")
                .padding(sides(12.0, 8.0))
                .background(Color::rgb(0.20, 0.35, 0.58))
                .radius(CornerRadii::all(8.0)),
            self.list.build("perf-million-variable", self.offset, row),
        ])
    }
}

fn row(index: usize) -> Element {
    Element::text(format!("row {index:07}"))
        .keyed(format!("perf-row-{index}"))
        .padding(sides(10.0, 4.0))
        .text_style(text(14.0, TextColor::rgb(0.76, 0.82, 0.91), 500))
}

fn panel(children: impl IntoIterator<Item = Element>) -> Element {
    Element::column(children)
        .padding(Sides::length(16.0))
        .gap(10.0)
        .background(Color::rgb(0.07, 0.085, 0.12))
        .border(Border::all(1.0, Color::rgb(0.16, 0.20, 0.28)))
        .radius(CornerRadii::all(14.0))
}

fn text(size: f32, color: TextColor, weight: u16) -> TextStyle {
    TextStyle {
        font_size: size,
        color,
        weight,
        wrap: TextWrap::None,
        ..TextStyle::default()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    use argui::{
        platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
        render::RendererConfig,
        runtime::run_app,
    };
    run_app(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui performance laboratory"),
            WindowConfig {
                title: "Argui performance laboratory".into(),
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        PerfShowcase::default(),
        |_| {},
    )
    .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use argui::{
        core::Point,
        runtime::{Context, Render},
        ui::{Element, UiEvent, UiEventKind, UiTree},
    };

    use super::PerfShowcase;

    fn event(key: &str, kind: UiEventKind) -> UiEvent {
        UiEvent {
            target: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            key: Some(key.into()),
            kind,
        }
    }

    #[test]
    fn labs_render_and_isolate_local_mutations() {
        let mut app = PerfShowcase::default();
        let mut cx = Context::default();
        let first = app.render(&mut cx);
        assert_eq!(first.children.len(), 5);
        assert_eq!(app.counter.read(|lab| lab.renders), 1);
        assert_eq!(app.fixed.read(|lab| lab.renders), 1);
        assert_eq!(app.variable.read(|lab| lab.renders), 1);

        app.event(&event("perf-counter", UiEventKind::Clicked), &mut cx);
        let _ = app.render(&mut cx);
        assert_eq!(app.counter.read(|lab| lab.value), 1);
        assert_eq!(app.fixed.read(|lab| lab.renders), 1);
        assert_eq!(app.variable.read(|lab| lab.renders), 1);
    }

    #[test]
    fn both_million_row_labs_keep_bounded_visible_windows() {
        let mut app = PerfShowcase::default();
        let mut cx = Context::default();
        let _ = app.render(&mut cx);
        app.event(
            &event(
                "perf-million-fixed",
                UiEventKind::Scrolled {
                    delta: Point::new(0.0, 20_000.0),
                    offset: Point::new(0.0, 20_000.0),
                },
            ),
            &mut cx,
        );
        app.event(
            &event(
                "perf-million-variable",
                UiEventKind::Scrolled {
                    delta: Point::new(0.0, 24_000.0),
                    offset: Point::new(0.0, 24_000.0),
                },
            ),
            &mut cx,
        );
        app.event(
            &event("perf-measure-variable", UiEventKind::Clicked),
            &mut cx,
        );
        let tree = app.render(&mut cx);
        assert_eq!(tree.children.len(), 5);
        assert!(
            app.fixed
                .read(|lab| lab.list.window(lab.offset).range.len())
                < 64
        );
        assert!(
            app.variable
                .read(|lab| lab.list.window(lab.offset).range.len())
                < 64
        );
    }
}
