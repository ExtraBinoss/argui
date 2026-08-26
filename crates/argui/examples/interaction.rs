use argui::{
    paint::{Border, ClipBehavior, Color, CornerRadii, PaintStyle, QuadStyle},
    platform::WindowConfig,
    render::RendererConfig,
    runtime::{RuntimeEvent, run_ui},
    text::{TextColor, TextStyle},
    ui::{Align, Button, ButtonStyle, Edges, Element, Length, UiEventKind, UiTree, Wrap},
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

fn button(key: &str, label: &str, accent: Color) -> Element {
    let radius = CornerRadii::all(12.0);
    let rest = QuadStyle::solid(Color::rgb(0.10, 0.14, 0.20))
        .border(Border::all(1.0, Color::rgb(0.24, 0.32, 0.44)))
        .radius(radius);
    let hovered = QuadStyle::solid(Color::rgb(0.14, 0.22, 0.30))
        .border(Border::all(1.0, accent))
        .radius(radius);
    let pressed = QuadStyle::solid(accent)
        .border(Border::all(1.0, accent))
        .radius(radius)
        .opacity(0.82);
    let focused = rest.border(Border::all(2.0, accent));
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(rest),
            text_style(17.0, TextColor::WHITE, 600),
        )
        .hovered(hovered)
        .pressed(pressed)
        .focused(focused),
    )
    .build()
}

fn showcase() -> UiTree {
    UiTree::new(
        Element::column([Element::column([
            Element::text("Argui interaction")
                .text_style(text_style(44.0, TextColor::rgb(0.35, 0.85, 1.0), 700)),
            Element::text(
                "Hover, press, drag outside, and release. Hit testing follows the clipped Taffy geometry; pointer capture keeps the pressed target stable.",
            )
            .text_style(text_style(20.0, TextColor::WHITE, 400)),
            Element::row([
                button("primary", "Primary action", Color::rgb(0.15, 0.62, 0.92)),
                button("confirm", "Confirm", Color::rgb(0.22, 0.72, 0.46)),
                button("danger", "Delete", Color::rgb(0.92, 0.30, 0.36)),
            ])
            .wrap(Wrap::Wrap)
            .gap(12.0)
            .align(Align::Center),
            Element::text("The renderer only sees ordered quads and text.")
                .text_style(text_style(15.0, TextColor::rgb(0.62, 0.72, 0.84), 500)),
        ])
        .width(Length::Percent(1.0))
        .padding(Edges::all(30.0))
        .gap(24.0)
        .background(Color::rgb(0.06, 0.08, 0.12))
        .border(Border::all(1.5, Color::rgb(0.18, 0.28, 0.40)))
        .radius(CornerRadii::all(22.0))
        .clip(ClipBehavior::Bounds)])
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .align(Align::Center)
        .padding(Edges::symmetric(42.0, 36.0)),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_ui(
        WindowConfig {
            title: "Argui interaction showcase".into(),
            width: 1100.0,
            height: 700.0,
            ..WindowConfig::default()
        },
        RendererConfig::default(),
        showcase(),
        |event| {
            if let RuntimeEvent::Ui(event) = &event
                && event.kind == UiEventKind::Clicked
            {
                println!("clicked {:?}", event.key);
            }
            println!("{event:?}");
        },
    )?;
    Ok(())
}
