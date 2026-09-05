use argui::{
    paint::{Border, Color, CornerRadii, PaintStyle, QuadStyle},
    platform::WindowConfig,
    render::RendererConfig,
    runtime::{RuntimeEvent, run_ui},
    text::{TextColor, TextStyle},
    ui::{
        AlignItems, Axes, Element, FlexWrap, Overflow, Sides, UiEventKind, UiTree, percent, sides,
    },
};
use argui_widgets::{Button, ButtonStyle};

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
    let rest = QuadStyle::solid(Color::srgb(0.10, 0.14, 0.20))
        .border(Border::all(1.0, Color::srgb(0.24, 0.32, 0.44)))
        .radius(radius);
    let hovered = QuadStyle::solid(Color::srgb(0.14, 0.22, 0.30))
        .border(Border::all(1.0, accent))
        .radius(radius);
    let pressed = QuadStyle::solid(accent)
        .border(Border::all(1.0, accent))
        .radius(radius)
        .opacity(0.82);
    let focused = rest.clone().border(Border::all(2.0, accent));
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
                .text_style(text_style(44.0, TextColor::srgb(0.35, 0.85, 1.0), 700)),
            Element::text(
                "Hover, press, drag outside, and release. Hit testing follows the clipped Taffy geometry; pointer capture keeps the pressed target stable.",
            )
            .text_style(text_style(20.0, TextColor::WHITE, 400)),
            Element::row([
                button("primary", "Primary action", Color::srgb(0.15, 0.62, 0.92)),
                button("confirm", "Confirm", Color::srgb(0.22, 0.72, 0.46)),
                button("danger", "Delete", Color::srgb(0.92, 0.30, 0.36)),
            ])
            .flex_wrap(FlexWrap::Wrap)
            .gap(12.0)
            .align_items(AlignItems::CENTER),
            Element::text("The renderer only sees ordered quads and text.")
                .text_style(text_style(15.0, TextColor::srgb(0.62, 0.72, 0.84), 500)),
        ])
        .width(percent(1.0))
        .padding(Sides::length(30.0))
        .gap(24.0)
        .background(Color::srgb(0.06, 0.08, 0.12))
        .border(Border::all(1.5, Color::srgb(0.18, 0.28, 0.40)))
        .radius(CornerRadii::all(22.0))
        .overflow(Axes { x: Overflow::Hidden, y: Overflow::Hidden })])
        .width(percent(1.0))
        .height(percent(1.0))
        .align_items(AlignItems::CENTER)
        .padding(sides(42.0, 36.0)),
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
                && event.kind == UiEventKind::Click(argui::ui::ClickEvent::accessibility())
            {
                println!("clicked {:?}", event.target_key());
            }
            println!("{event:?}");
        },
    )?;
    Ok(())
}
