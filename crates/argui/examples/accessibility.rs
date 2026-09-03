use argui::{
    accessibility::{Role, SemanticAction, SemanticValue, Semantics},
    core::{Key, KeyState},
    paint::{Border, Color, CornerRadii, PaintStyle, QuadStyle},
    platform::{ApplicationConfig, ApplicationIdentity, PlatformEvent, WindowConfig},
    render::RendererConfig,
    runtime::{Context, Render, RuntimeEvent, run_app},
    text::{TextColor, TextStyle},
    ui::{
        AlignItems, Element, FocusScope, GestureSet, InitialFocus, Interaction, Sides, UiEvent,
        UiEventKind, length, percent,
    },
};
use argui_widgets::{Button, ButtonStyle, Input, InputStyle};

fn text_style(size: f32, color: TextColor, weight: u16) -> TextStyle {
    TextStyle {
        font_size: size,
        line_height: size * 1.25,
        color,
        weight,
        ..TextStyle::default()
    }
}

fn panel() -> PaintStyle {
    PaintStyle::new(
        QuadStyle::solid(Color::srgb(0.075, 0.095, 0.135))
            .border(Border::all(1.0, Color::srgb(0.20, 0.30, 0.44)))
            .radius(CornerRadii::all(18.0)),
    )
}

fn button(key: &str, label: &str) -> Element {
    let rest = QuadStyle::solid(Color::srgb(0.10, 0.25, 0.38))
        .border(Border::all(1.0, Color::srgb(0.25, 0.75, 0.95)))
        .radius(CornerRadii::all(10.0));
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(rest.clone()),
            text_style(17.0, TextColor::WHITE, 650),
        )
        .hovered(rest.clone().opacity(0.86))
        .pressed(rest.clone().opacity(0.68))
        .focused(rest.border(Border::all(2.0, Color::WHITE))),
    )
    .build()
}

fn input() -> Element {
    let rest = QuadStyle::solid(Color::srgb(0.055, 0.07, 0.10))
        .border(Border::all(1.0, Color::srgb(0.24, 0.34, 0.48)))
        .radius(CornerRadii::all(10.0));
    Input::new(
        "name",
        "",
        "Your name",
        InputStyle::new(
            PaintStyle::new(rest.clone()),
            text_style(17.0, TextColor::WHITE, 450),
        )
        .hovered(
            rest.clone()
                .border(Border::all(1.0, Color::srgb(0.35, 0.75, 0.95))),
        )
        .focused(rest.border(Border::all(2.0, Color::WHITE))),
    )
    .build()
}

fn gesture_surface() -> Element {
    Element::column([
        Element::text("Touch laboratory")
            .text_style(text_style(20.0, TextColor::WHITE, 650))
            .semantic_hidden(true),
        Element::text("Tap, drag, pinch or rotate with two fingers")
            .text_style(text_style(15.0, TextColor::srgb(0.68, 0.78, 0.90), 450))
            .semantic_hidden(true),
    ])
    .keyed("gesture-surface")
    .width(percent(1.0))
    .padding(Sides::length(20.0))
    .gap(7.0)
    .paint_style(panel())
    .interaction(
        Interaction::default()
            .focusable(true)
            .gestures(GestureSet::ALL),
    )
    .semantics(
        Semantics::new(Role::Slider)
            .label("Gesture laboratory")
            .description("Accepts tap, pan, pinch and rotation")
            .value(SemanticValue::Number {
                value: 50.0,
                minimum: Some(0.0),
                maximum: Some(100.0),
                step: Some(1.0),
            })
            .action(SemanticAction::Focus)
            .action(SemanticAction::Increment)
            .action(SemanticAction::Decrement),
    )
}

fn showcase(dialog_open: bool) -> Element {
    let mut content = vec![Element::column([
            Element::text("Accessibility and touch")
                .text_style(text_style(
                    38.0,
                    TextColor::srgb(0.35, 0.85, 1.0),
                    750,
                ))
                .semantics(
                    Semantics::new(Role::Heading)
                        .label("Accessibility and touch")
                        .level(1),
                ),
            Element::text(
                "Use Tab, Enter, a screen reader, touch, pen or mouse. The GPU renderer never owns these semantics.",
            )
            .text_style(text_style(17.0, TextColor::WHITE, 450)),
            input(),
            button("announce", "Accessible action"),
            button("open-dialog", "Open modal dialog"),
            gesture_surface(),
        ])
        .width(percent(1.0))
        .max_width(length(720.0))
        .padding(Sides::length(28.0))
        .gap(18.0)
        .paint_style(panel())];
    if dialog_open {
        content.push(
            Element::column([
                Element::text("Keyboard focus is trapped here").text_style(text_style(
                    22.0,
                    TextColor::WHITE,
                    700,
                )),
                Element::text("Tab cycles inside. Escape or the button closes and restores focus.")
                    .text_style(text_style(16.0, TextColor::WHITE, 450)),
                button("dismiss-dialog", "Dismiss dialog"),
            ])
            .keyed("modal-dialog")
            .width(length(460.0))
            .padding(Sides::length(24.0))
            .gap(16.0)
            .paint_style(panel())
            .z_index(100)
            .focus_scope(FocusScope::modal(InitialFocus::Target(
                "dismiss-dialog".into(),
            )))
            .semantics(Semantics::new(Role::Dialog).label("Focus demonstration")),
        );
    }
    Element::column(content)
        .width(percent(1.0))
        .height(percent(1.0))
        .align_items(AlignItems::CENTER)
        .padding(Sides::length(32.0))
}

#[derive(Default)]
struct AccessibilityDemo {
    dialog_open: bool,
}

impl Render for AccessibilityDemo {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        showcase(self.dialog_open)
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let close_from_escape = matches!(
            &event.kind,
            UiEventKind::KeyInput(input)
                if input.key == Key::Escape
                    && input.state == KeyState::Pressed
                    && !input.repeat
                    && self.dialog_open
        );
        match (event.key.as_deref(), &event.kind) {
            (Some("open-dialog"), UiEventKind::Clicked) => {
                self.dialog_open = true;
                cx.notify();
            }
            (Some("dismiss-dialog"), UiEventKind::Clicked) if self.dialog_open => {
                self.dialog_open = false;
                cx.notify();
            }
            _ if close_from_escape => {
                self.dialog_open = false;
                cx.notify();
            }
            _ => {}
        }
        if matches!(
            event.kind,
            UiEventKind::Gesture(_)
                | UiEventKind::SemanticAction { .. }
                | UiEventKind::Clicked
                | UiEventKind::TextChanged(_)
                | UiEventKind::KeyInput(_)
        ) {
            println!(
                "{}: {:?}",
                event.key.as_deref().unwrap_or("semantic"),
                event.kind
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui accessibility and touch"),
            WindowConfig {
                title: "Argui accessibility and touch".into(),
                width: 900.0,
                height: 720.0,
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        AccessibilityDemo::default(),
        |event| {
            if let RuntimeEvent::Platform(PlatformEvent::PreferencesChanged(preferences)) = event {
                println!("system accessibility: {preferences:?}");
            }
        },
    )?;
    Ok(())
}
