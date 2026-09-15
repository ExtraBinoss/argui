//! Deterministic retained-tree stress labs, shared by native and WebAssembly.

use argui::{
    paint::{Border, Color, CornerRadii},
    runtime::{Context, Entity, Render},
    text::{TextColor, TextStyle, TextWrap},
    ui::{
        Axes, Element, EventType, Overflow, ScrollConfig, Sides, UiEventKind, VirtualList, percent,
        sides,
    },
};

const ROWS: usize = 1_000_000;

/// Retained-tree performance laboratory with counter and virtual-list examples.
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
            .text_style(text(15.0, TextColor::srgb(0.48, 0.88, 0.68), 650)),
            cx.entity(&self.counter),
            cx.entity(&self.fixed),
            cx.entity(&self.variable),
        ])
        .padding(Sides::length(24.0))
        .gap(18.0)
        .width(percent(1.0))
        .height(percent(1.0))
        .background(Color::srgb(0.035, 0.045, 0.065))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default())
    }
}

#[derive(Default)]
struct CounterLab {
    value: u64,
    renders: u64,
}

impl Render for CounterLab {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        panel([
            Element::text("Local retained state").text_style(text(19.0, TextColor::WHITE, 700)),
            Element::text(format!("value={} (only this entity changes)", self.value))
                .keyed("perf-counter")
                .on(cx.listener(EventType::Click, |lab, _, cx| {
                    lab.value += 1;
                    cx.notify();
                }))
                .padding(sides(14.0, 9.0))
                .background(Color::srgb(0.12, 0.36, 0.28))
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
            list: VirtualList::fixed(ROWS, 28.0, 240.0).overscan(4),
            offset: 0.0,
            renders: 0,
        }
    }
}

impl Render for FixedListLab {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        let list = self
            .list
            .clone()
            .build("perf-million-fixed", self.offset, row)
            .on(cx.listener(EventType::Scroll, |lab, event, cx| {
                let UiEventKind::Scrolled { offset, .. } = event.kind else {
                    return;
                };
                let changed = lab.list.window(lab.offset).range != lab.list.window(offset.y).range;
                lab.offset = offset.y;
                if changed {
                    cx.notify();
                }
            }));
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
    list: VirtualList,
    offset: f32,
    renders: u64,
}

impl Default for VariableListLab {
    fn default() -> Self {
        Self {
            list: VirtualList::variable(ROWS, 34.0, 260.0).overscan(5),
            offset: 0.0,
            renders: 0,
        }
    }
}

impl Render for VariableListLab {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        panel([
            Element::text("1,000,000 variable-height rows · anchored measurements")
                .text_style(text(19.0, TextColor::WHITE, 700)),
            Element::text("Visible rows are measured automatically; the anchor stays fixed.")
                .text_style(text(14.0, TextColor::srgb(0.62, 0.70, 0.82), 500)),
            self.list
                .build("perf-million-variable", self.offset, variable_row)
                .on(cx.listener(EventType::Scroll, |lab, event, cx| {
                    let UiEventKind::Scrolled { offset, .. } = event.kind else {
                        return;
                    };
                    let changed =
                        lab.list.window(lab.offset).range != lab.list.window(offset.y).range;
                    lab.offset = offset.y;
                    if changed {
                        cx.notify();
                    }
                })),
        ])
    }
}

fn row(index: usize) -> Element {
    Element::text(format!("row {index:07}"))
        .keyed(format!("perf-row-{index}"))
        .padding(sides(10.0, 4.0))
        .text_style(text(14.0, TextColor::srgb(0.76, 0.82, 0.91), 500))
}

fn variable_row(index: usize) -> Element {
    Element::text(format!(
        "row {index:07}{}",
        if index.is_multiple_of(3) {
            " · expanded content"
        } else {
            ""
        }
    ))
    .keyed(format!("perf-variable-row-{index}"))
    .padding(sides(10.0, 5.0 + (index % 4) as f32 * 4.0))
    .text_style(text(14.0, TextColor::srgb(0.76, 0.82, 0.91), 500))
}

fn panel(children: impl IntoIterator<Item = Element>) -> Element {
    Element::column(children)
        .padding(Sides::length(16.0))
        .gap(10.0)
        .background(Color::srgb(0.07, 0.085, 0.12))
        .border(Border::all(1.0, Color::srgb(0.16, 0.20, 0.28)))
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
/// Starts the performance laboratory in a WebAssembly browser runtime.
///
/// # Errors
///
/// Returns a JavaScript error if runtime startup fails.
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
