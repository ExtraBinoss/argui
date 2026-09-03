use argui::{
    core::{Color, ColorInterpolation, Point, Rect},
    paint::{
        Border, CornerRadii, Fill, GradientStop, LayerMask, LayerStyle, LinearGradient, Shadow,
    },
    text::TextAlign,
    ui::{
        AlignItems, Element, FlexWrap, JustifyContent, Resizable, Sides, evenly_sized_tracks,
        length, percent,
    },
    widgets::{
        Button, Checkbox, Dialog, DialogBehavior, Input, RadioGroup, RadioOption, RangeConfig,
        Select, SelectOption, Slider, Switch, Tab, TablerIcon, Tabs, TextArea, WidgetAssets,
        WidgetTheme,
    },
};
use argui_effects::AnimatedGradient;

use crate::{
    app::{WidgetGallery, text},
    navigation::Page,
};

mod buttons;
mod composition;
mod inputs;
mod typography;

pub(crate) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    spinner: Element,
) -> Element {
    let content = match gallery.page {
        Page::Button => buttons::render(gallery, theme, spinner),
        Page::Input => inputs::render(gallery, theme, assets),
        Page::TextArea => textarea(gallery, theme, assets),
        Page::Checkbox => checkboxes(gallery, theme, assets),
        Page::Switch => switches(gallery, theme),
        Page::RadioGroup => radios(gallery, theme),
        Page::Slider => sliders(gallery, theme, assets),
        Page::Tabs => tabs(gallery, theme),
        Page::Select => selects(gallery, theme, assets),
        Page::Dialog => dialogs(gallery, theme),
        Page::Form => form(gallery, theme, assets),
        Page::Settings => settings(gallery, theme),
        Page::Layout => layout_system(theme),
        Page::Motion => motion(theme, spinner),
        Page::Effects => effects(theme),
        Page::Composition => composition::render(gallery, theme, assets),
        Page::Typography => typography::render(theme),
    };
    Element::column([
        Element::column([
            text(gallery.page.label(), 30.0, theme.foreground, 740),
            text(description(gallery.page), 15.0, theme.muted_foreground, 400),
        ])
        .gap(5.0),
        content,
    ])
    .width(percent(1.0))
    .max_width(length(920.0))
    .gap(24.0)
}

fn textarea(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    let mut style = theme.input.clone();
    style.layout.size.height = percent(1.0);
    style.layout.padding = Sides::length(13.0);
    let scrollbar = theme.scrollbar.clone().insets(Sides {
        bottom: 22.0,
        ..Sides::length(4.0)
    });
    let area = TextArea::new("notes", &gallery.notes, "Write a note…", style)
        .scrollbar(scrollbar)
        .build();
    preview(
        "Resizable multiline input",
        "The engine preserves caret and scroll while the captured resize gesture changes layout.",
        Resizable::new(
            gallery.editor_size.size(),
            area,
            "notes-resize",
            assets.icon(TablerIcon::Resize, 18.0),
        )
        .build(),
        theme,
    )
}

fn checkboxes(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    preview(
        "Checkbox states",
        "Space and Enter activate the focused control.",
        Element::column([
            Checkbox::new("accepted", "Accept the renderer terms", gallery.accepted)
                .indicator(assets.icon(TablerIcon::Check, 14.0))
                .build(theme),
            Checkbox::new("check-empty", "Unchecked option", false).build(theme),
            Checkbox::new("check-disabled", "Disabled option", true)
                .enabled(false)
                .build(theme),
        ])
        .gap(8.0),
        theme,
    )
}

fn switches(gallery: &WidgetGallery, theme: &WidgetTheme) -> Element {
    preview(
        "Switch states",
        "The thumb transition is built from the same retained state model as every element.",
        Element::column([
            Switch::new("notifications", "GPU profiling", gallery.notifications).build(theme),
            Switch::new("switch-off", "Offline rendering", false).build(theme),
            Switch::new("switch-disabled", "Unavailable backend", true)
                .enabled(false)
                .build(theme),
        ])
        .gap(8.0),
        theme,
    )
}

fn radios(gallery: &WidgetGallery, theme: &WidgetTheme) -> Element {
    preview(
        "Quality preset",
        "Arrow navigation uses spatial focus and activation remains controlled.",
        RadioGroup::new(
            "quality",
            "Effect quality",
            [
                RadioOption::new("Maximum quality"),
                RadioOption::new("Balanced"),
                RadioOption::new("Performance"),
            ],
            Some(gallery.radio),
        )
        .build(theme),
        theme,
    )
}

fn sliders(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    preview(
        "Continuous and stepped input",
        "Drag, tap, arrow keys, Home/End and AccessKit values share one clamping path.",
        Element::column([
            crate::property_slider::render(gallery, theme, assets),
            Slider::new(
                "plain-slider",
                "Plain slider",
                gallery.plain_slider,
                RangeConfig::default(),
            )
            .build(theme),
        ])
        .gap(12.0),
        theme,
    )
}

fn tabs(gallery: &WidgetGallery, theme: &WidgetTheme) -> Element {
    preview(
        "Tabbed settings",
        "Only the selected panel is mounted and exposed to accessibility.",
        Tabs::new(
            "demo-tabs",
            [
                Tab::new(
                    "General",
                    panel_text("General rendering preferences", theme),
                ),
                Tab::new("Performance", panel_text("Frame pacing and caches", theme)),
                Tab::new("Advanced", panel_text("Custom WGSL registry", theme)),
            ],
            gallery.tab,
        )
        .build(theme),
        theme,
    )
}

fn selects(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    preview(
        "Backend selection",
        "The overlay follows its anchor, flips at viewport edges and restores focus on close.",
        Select::new(
            "backend",
            "Choose a backend",
            select_options(),
            gallery.select_selected,
        )
        .open(gallery.select_open)
        .highlighted(gallery.select_highlight)
        .trailing(assets.icon(TablerIcon::ChevronDown, 17.0))
        .build(theme),
        theme,
    )
}

fn dialogs(gallery: &WidgetGallery, theme: &WidgetTheme) -> Element {
    let content = Element::column([
        text("Delete GPU cache?", 22.0, theme.foreground, 700),
        text(
            "This demonstrates a modal focus trap, Escape handling and focus restoration.",
            14.0,
            theme.muted_foreground,
            400,
        ),
        Button::new(
            DialogBehavior::new("demo-dialog", "Delete GPU cache", gallery.dialog_open).close_key(),
            "Close dialog",
            theme.outline_button.clone(),
        )
        .build(),
    ])
    .gap(14.0);
    preview(
        "Modal dialog",
        "Background controls cannot receive focus while the modal scope is active.",
        Dialog::new(
            "demo-dialog",
            "Delete GPU cache",
            gallery.dialog_open,
            Button::new("open-dialog", "Open dialog", theme.button.clone()).build(),
            content,
        )
        .build(theme),
        theme,
    )
}

fn form(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    preview(
        "Profile form",
        "A realistic composition made only from public widget APIs.",
        Element::column([
            Input::new("name", &gallery.name, "Full name", theme.input.clone())
                .label("Full name")
                .build(),
            Input::new("email", &gallery.email, "Email", theme.input.clone())
                .label("Email")
                .build(),
            Select::new(
                "backend",
                "Preferred backend",
                select_options(),
                gallery.select_selected,
            )
            .open(gallery.select_open)
            .highlighted(gallery.select_highlight)
            .trailing(assets.icon(TablerIcon::ChevronDown, 17.0))
            .build(theme),
            Checkbox::new("accepted", "Share anonymous GPU metrics", gallery.accepted)
                .indicator(assets.icon(TablerIcon::Check, 14.0))
                .build(theme),
            Button::new("save-profile", "Save profile", theme.button.clone()).build(),
        ])
        .gap(12.0),
        theme,
    )
}

fn settings(gallery: &WidgetGallery, theme: &WidgetTheme) -> Element {
    preview(
        "Renderer settings",
        "Switches, radio controls, tabs and a slider remain independent controlled values.",
        Element::column([
            Switch::new("notifications", "Live GPU profiling", gallery.notifications).build(theme),
            RadioGroup::new(
                "quality",
                "Quality",
                [
                    RadioOption::new("Quality"),
                    RadioOption::new("Balanced"),
                    RadioOption::new("Performance"),
                ],
                Some(gallery.radio),
            )
            .orientation(argui::accessibility::Orientation::Horizontal)
            .build(theme),
            Slider::new(
                "property-slider",
                "Render scale",
                gallery.slider,
                RangeConfig::default(),
            )
            .build(theme),
        ])
        .gap(16.0),
        theme,
    )
}

fn layout_system(theme: &WidgetTheme) -> Element {
    let flex = Element::row([
        layout_tile("A", theme),
        layout_tile("B", theme),
        layout_tile("C", theme),
    ])
    .width(percent(1.0))
    .height(length(92.0))
    .padding(Sides::length(10.0))
    .gap(10.0)
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::SPACE_BETWEEN)
    .background(theme.muted)
    .radius(CornerRadii::all(10.0));
    let grid = Element::grid((1..=6).map(|index| layout_tile(index.to_string(), theme)))
        .grid_template_columns(evenly_sized_tracks::<String>(3))
        .width(percent(1.0))
        .padding(Sides::length(10.0))
        .gap(10.0)
        .background(theme.muted)
        .radius(CornerRadii::all(10.0));
    let text_alignment = Element::column([
        text("Logical start · مرحبا", 14.0, theme.foreground, 500).text_align(TextAlign::Start),
        text("Centered content", 14.0, theme.foreground, 500).text_align(TextAlign::Center),
        text("Logical end · שלום", 14.0, theme.foreground, 500).text_align(TextAlign::End),
    ])
    .width(percent(1.0))
    .padding(Sides::length(12.0))
    .gap(8.0)
    .background(theme.muted)
    .radius(CornerRadii::all(10.0));
    preview(
        "Block, Flex, Grid and text flow",
        "The public style maps directly to Taffy with CSS box sizing, intrinsic media and logical text alignment.",
        Element::column([flex, grid, text_alignment]).gap(14.0),
        theme,
    )
}

fn layout_tile(label: impl Into<String>, theme: &WidgetTheme) -> Element {
    Element::row([text(label, 14.0, theme.primary_foreground, 700)])
        .height(length(44.0))
        .padding(Sides::length(12.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)
        .background(theme.primary)
        .radius(CornerRadii::all(7.0))
}

fn motion(theme: &WidgetTheme, spinner: Element) -> Element {
    preview(
        "Motion and loading",
        "The spinner requests presentation frames only while mounted; reduced-motion stays still.",
        Element::row([
            Button::new("loading-example", "Compiling shaders", theme.button.clone())
                .loading(spinner)
                .build(),
            Button::new(
                "motion-hover",
                "Hover and press",
                theme.outline_button.clone(),
            )
            .build(),
        ])
        .flex_wrap(FlexWrap::Wrap)
        .gap(12.0),
        theme,
    )
}

fn effects(theme: &WidgetTheme) -> Element {
    let gradient = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        ColorInterpolation::Oklab,
        [
            GradientStop::new(0.0, theme.primary),
            GradientStop::new(1.0, Color::srgb(0.65, 0.20, 0.94)),
        ],
    )
    .expect("the gallery gradient stops are sorted");
    let custom = Element::container([
        text("Custom WGSL", 20.0, Color::WHITE, 700),
        text(
            "Registered by the opt-in argui-effects crate",
            13.0,
            Color::WHITE,
            500,
        ),
    ])
    .width(percent(1.0))
    .height(length(190.0))
    .padding(Sides::length(22.0))
    .gap(7.0)
    .fill(Fill::Linear(gradient))
    .radius(CornerRadii::all(16.0))
    .layer(
        LayerStyle::new(Rect::default())
            .filter(AnimatedGradient::new(0.28).filter())
            .shadow(Shadow::drop(
                [0.0, 14.0],
                32.0,
                Color::srgba(0.0, 0.0, 0.0, 0.28),
            ))
            .mask(LayerMask::Rounded(CornerRadii::all(16.0))),
    );
    preview(
        "Opt-in GPU effect registry",
        "The gallery registers presets and renders every authored color through the same linear GPU pipeline.",
        Element::column([custom, color_spaces(theme)]).gap(18.0),
        theme,
    )
}

fn color_spaces(theme: &WidgetTheme) -> Element {
    let samples = [
        ("OKLab", ColorInterpolation::Oklab),
        ("linear sRGB", ColorInterpolation::LinearSrgb),
        ("sRGB", ColorInterpolation::Srgb),
    ];
    Element::column(samples.map(|(label, interpolation)| {
        let gradient = LinearGradient::new(
            Point::default(),
            Point::new(1.0, 0.0),
            interpolation,
            [
                GradientStop::new(0.0, Color::from_srgb8(239, 68, 68)),
                GradientStop::new(1.0, Color::from_srgb8(59, 130, 246)),
            ],
        )
        .expect("the color-space sample stops are sorted");
        Element::row([
            text(label, 12.0, theme.muted_foreground, 600).width(length(92.0)),
            Element::container([])
                .grow(1.0)
                .height(length(28.0))
                .fill(Fill::Linear(gradient))
                .radius(CornerRadii::all(7.0)),
        ])
        .align_items(AlignItems::CENTER)
        .gap(12.0)
    }))
    .gap(8.0)
}

fn preview(title: &str, description: &str, content: Element, theme: &WidgetTheme) -> Element {
    Element::column([
        Element::column([
            text(title, 17.0, theme.foreground, 650),
            text(description, 13.0, theme.muted_foreground, 400),
        ])
        .gap(3.0),
        content,
        Element::container([])
            .width(percent(1.0))
            .height(length(1.0))
            .background(theme.border),
    ])
    .width(percent(1.0))
    .gap(22.0)
}

fn panel_text(label: &str, theme: &WidgetTheme) -> Element {
    text(label, 14.0, theme.muted_foreground, 500)
        .padding(Sides::length(18.0))
        .background(theme.background)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(8.0))
}

fn select_options() -> Vec<SelectOption> {
    ["Vulkan", "DirectX 12", "Metal", "WebGPU"]
        .into_iter()
        .map(SelectOption::new)
        .collect()
}

const fn description(page: Page) -> &'static str {
    match page {
        Page::Button => "Actions with variants, icons, loading and accessible activation.",
        Page::Input => "Controlled single-line and search fields.",
        Page::TextArea => "Multiline editing, scrolling, clipping and resize capture.",
        Page::Checkbox => "Boolean state with keyboard and touch activation.",
        Page::Switch => "Animated binary preferences.",
        Page::RadioGroup => "Exclusive selection with semantic grouping.",
        Page::Slider => "Pointer, touch, keyboard and accessibility values.",
        Page::Tabs => "Roving navigation and one mounted panel.",
        Page::Select => "Anchored, collision-aware option overlay.",
        Page::Dialog => "Modal focus containment and restoration.",
        Page::Form => "A complete controlled profile form.",
        Page::Settings => "A realistic renderer configuration panel.",
        Page::Layout => "CSS-shaped Block, Flex, Grid, box model and text alignment.",
        Page::Motion => "Frame-paced feedback and interaction transitions.",
        Page::Effects => "Custom WGSL through the generic effect registry.",
        Page::Composition => "Compound state, responsive layout and exact hit geometry.",
        Page::Typography => "Rich spans, decoration, clamping and web-like text selection.",
    }
}
