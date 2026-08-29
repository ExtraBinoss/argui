use argui::{
    accessibility::{Role, SemanticAction, SemanticValue, Semantics},
    paint::{Border, Color, CornerRadii, PaintStyle, QuadStyle},
    platform::{PlatformEvent, WindowConfig},
    render::RendererConfig,
    runtime::{RuntimeEvent, run_ui},
    text::{TextColor, TextStyle},
    ui::{
        Align, Button, ButtonStyle, Edges, Element, GestureSet, Interaction, Length, TextInput,
        TextInputStyle, UiEventKind, UiTree,
    },
};

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
        QuadStyle::solid(Color::rgb(0.075, 0.095, 0.135))
            .border(Border::all(1.0, Color::rgb(0.20, 0.30, 0.44)))
            .radius(CornerRadii::all(18.0)),
    )
}

fn button() -> Element {
    let rest = QuadStyle::solid(Color::rgb(0.10, 0.25, 0.38))
        .border(Border::all(1.0, Color::rgb(0.25, 0.75, 0.95)))
        .radius(CornerRadii::all(10.0));
    Button::new(
        "announce",
        "Accessible action",
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
    let rest = QuadStyle::solid(Color::rgb(0.055, 0.07, 0.10))
        .border(Border::all(1.0, Color::rgb(0.24, 0.34, 0.48)))
        .radius(CornerRadii::all(10.0));
    TextInput::new(
        "name",
        "",
        "Your name",
        TextInputStyle::new(
            PaintStyle::new(rest.clone()),
            text_style(17.0, TextColor::WHITE, 450),
        )
        .hovered(
            rest.clone()
                .border(Border::all(1.0, Color::rgb(0.35, 0.75, 0.95))),
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
            .text_style(text_style(15.0, TextColor::rgb(0.68, 0.78, 0.90), 450))
            .semantic_hidden(true),
    ])
    .keyed("gesture-surface")
    .width(Length::Percent(1.0))
    .padding(Edges::all(20.0))
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

fn showcase() -> UiTree {
    UiTree::new(
        Element::column([Element::column([
            Element::text("Accessibility and touch")
                .text_style(text_style(
                    38.0,
                    TextColor::rgb(0.35, 0.85, 1.0),
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
            button(),
            gesture_surface(),
        ])
        .width(Length::Percent(1.0))
        .max_width(Length::Px(720.0))
        .padding(Edges::all(28.0))
        .gap(18.0)
        .paint_style(panel())])
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .align(Align::Center)
        .padding(Edges::all(32.0)),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_ui(
        WindowConfig {
            title: "Argui accessibility and touch".into(),
            width: 900.0,
            height: 720.0,
            ..WindowConfig::default()
        },
        RendererConfig::default(),
        showcase(),
        |event| match event {
            RuntimeEvent::Ui(event)
                if matches!(
                    event.kind,
                    UiEventKind::Gesture(_)
                        | UiEventKind::SemanticAction { .. }
                        | UiEventKind::Clicked
                        | UiEventKind::TextChanged(_)
                ) =>
            {
                println!(
                    "{}: {:?}",
                    event.key.as_deref().unwrap_or("semantic"),
                    event.kind
                );
            }
            RuntimeEvent::Platform(PlatformEvent::AccessibilityPreferences(preferences)) => {
                println!("system accessibility: {preferences:?}");
            }
            _ => {}
        },
    )?;
    Ok(())
}
