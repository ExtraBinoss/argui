use argui::{
    core::{Color, ColorInterpolation, Point, Rect},
    paint::{
        Border, CornerRadii, Fill, GradientStop, LayerMask, LayerStyle, LinearGradient, Shadow,
    },
    text::TextAlign,
    ui::{
        AlignItems, CursorIcon, Element, EventListener, FlexWrap, GestureCapture, GestureSet,
        Interaction, JustifyContent, PanGesture, Position, Sides, auto, evenly_sized_tracks,
        length, percent,
    },
    widgets::{
        Button, Checkbox, Dialog, DialogBehavior, RadioGroup, RadioOption, RangeConfig, Select,
        SelectOption, Slider, Switch, Tab, TablerIcon, Tabs, TextArea, WidgetAssets, WidgetTheme,
    },
};
use argui_effects::AnimatedGradient;

use crate::{
    app::{WidgetGallery, text},
    navigation::Page,
};

mod action_menu;
pub(crate) mod actions;
mod alert;
mod aspect_ratio;
pub(crate) mod async_tasks;
mod avatar;
mod badge;
mod buttons;
mod card;
mod collapsible;
pub(crate) mod data;
pub(crate) mod data_table;
pub(crate) mod dates;
pub(crate) mod editing;
mod empty;
mod inputs;
mod kbd;
pub(crate) mod liquid_glass;
pub(crate) mod menus;
pub(crate) mod progress;
pub(crate) mod scroll_effects;
mod separator;
pub(crate) mod timeline;
pub(crate) mod toast;
mod typography;
pub(crate) mod webview;

pub(crate) struct ResizeListeners {
    pub(crate) textarea: EventListener,
    pub(crate) textarea_reset: EventListener,
}

pub(crate) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut argui::runtime::Context<WidgetGallery>,
    resize: ResizeListeners,
) -> Element {
    let content = match gallery.page {
        Page::Menu | Page::ContextMenu | Page::Menubar => {
            menus::render(&gallery.menus, gallery.page, cx)
        }
        Page::Calendar | Page::DatePicker => {
            dates::render(&gallery.dates, gallery.page == Page::DatePicker, cx)
        }
        Page::DataTable => cx.entity(
            gallery
                .data_table
                .get_or_init(|| argui::runtime::Entity::new(data_table::TableDemo::new(assets))),
        ),
        Page::Toast => toast::controls(&gallery.toasts, theme, cx),
        Page::List | Page::VList | Page::Table => data::render(&gallery.data, gallery.page, cx),
        Page::Avatar => avatar::render(theme, gallery.logo),
        Page::Empty => empty::render(theme, assets, cx),
        Page::Kbd => kbd::render(theme),
        Page::AspectRatio => aspect_ratio::render(theme, gallery.logo),
        Page::Progress => cx.entity(
            gallery
                .progress
                .get_or_init(|| argui::runtime::Entity::new(progress::ProgressDemo::default())),
        ),
        Page::Badge => badge::render(theme, assets),
        Page::Card => card::render(theme, assets, cx),
        Page::Alert => alert::render(theme, assets),
        Page::Separator => separator::render(theme),
        Page::Collapsible => collapsible::render(gallery, theme, assets, cx),
        Page::Button => buttons::render(gallery, theme, cx.entity(&gallery.spinner)),
        Page::Input => inputs::render(gallery, theme, assets),
        Page::TextArea => textarea(
            gallery,
            theme,
            assets,
            resize.textarea,
            resize.textarea_reset,
        ),
        Page::Checkbox => checkboxes(gallery, theme, assets),
        Page::Switch => switches(gallery, theme),
        Page::RadioGroup => radios(gallery, theme),
        Page::Slider => sliders(gallery, theme, assets),
        Page::Tabs => tabs(gallery, theme),
        Page::Select => selects(gallery, theme, assets),
        Page::Dialog => dialogs(gallery, theme),
        Page::Layout => layout_system(theme),
        Page::Motion => motion(theme, cx.entity(&gallery.spinner)),
        Page::Effects => Element::column([
            cx.entity(&gallery.scroll_demo),
            effects(theme),
            cx.entity(&gallery.glass),
        ])
        .gap(24.0),
        Page::Typography => typography::render(theme),
        Page::WebView => cx.entity(&gallery.webview),
        Page::AsyncTasks => cx.entity(&gallery.tasks),
        Page::Actions => cx.entity(&gallery.actions),
        Page::Editing => cx.entity(&gallery.editing),
        Page::CustomTimeline => cx.entity(&gallery.timeline),
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
    .max_width(if gallery.page == Page::Effects {
        percent(1.0)
    } else {
        length(920.0)
    })
    .gap(24.0)
}

fn textarea(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    resize_listener: EventListener,
    resize_reset_listener: EventListener,
) -> Element {
    let mut style = theme.input();
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
        resizable_textarea(
            gallery.editor_size,
            area,
            assets,
            resize_listener,
            resize_reset_listener,
        ),
        theme,
    )
}

fn resizable_textarea(
    size: argui::core::Size,
    area: Element,
    assets: &WidgetAssets,
    listener: EventListener,
    reset_listener: EventListener,
) -> Element {
    let handle = assets
        .icon(TablerIcon::Resize, 18.0)
        .keyed("notes-resize")
        .absolute(Sides {
            left: auto(),
            right: length(0.0),
            top: auto(),
            bottom: length(0.0),
        })
        .interaction(
            Interaction::default()
                .cursor(CursorIcon::NwseResize)
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress)
                            .delivery(argui::ui::GestureDelivery::FrameCoalesced),
                    ),
                ),
        )
        .on(listener)
        .on(reset_listener);
    Element::container([area, handle])
        .keyed("notes-resizable")
        .width(length(size.width))
        .height(length(size.height))
        .position(Position::Relative)
}

fn checkboxes(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    preview(
        "Checkbox states",
        "Space and Enter activate the focused control.",
        Element::column([
            Checkbox::new(
                "accepted",
                "Accept the renderer terms",
                if gallery.accepted {
                    argui::ui::CheckedState::Checked
                } else {
                    argui::ui::CheckedState::Unchecked
                },
            )
            .indicator(assets.icon(TablerIcon::Check, 14.0))
            .build(theme),
            Checkbox::new(
                "check-empty",
                "Unchecked option",
                argui::ui::CheckedState::Unchecked,
            )
            .build(theme),
            Checkbox::new(
                "check-disabled",
                "Disabled option",
                argui::ui::CheckedState::Checked,
            )
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
        .presence(&gallery.select_presence)
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
            theme.outline_button(),
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
            Button::new("open-dialog", "Open dialog", theme.button()).build(),
            content,
        )
        .build(theme),
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
            Button::new("loading-example", "Compiling shaders", theme.button())
                .loading(spinner)
                .build(),
            Button::new("motion-hover", "Hover and press", theme.outline_button()).build(),
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
        Page::List => "Selection and keyboard navigation.",
        Page::VList => "Measured variable-height rows and virtual scrolling.",
        Page::Table => "Columns and row selection.",
        Page::Menu => "Nested menus, checkbox and radio entries.",
        Page::ContextMenu => "Open a menu at the pointer or with Shift+F10.",
        Page::Menubar => "Move between menus with the arrow keys.",
        Page::DataTable => "Sort columns, select rows and edit cells with Enter.",
        Page::Calendar => "Choose dates with arrows and PageUp/PageDown.",
        Page::DatePicker => "Type a date or choose it from the calendar.",
        Page::Toast => "Notifications with a bounded queue and explicit dismissal.",
        Page::Avatar => "Images and initials with a shared accessible name.",
        Page::Empty => "Give an empty view a useful message and a next step.",
        Page::Kbd => "Keycaps and shortcut combinations.",
        Page::AspectRatio => "Keep content at a consistent width-to-height ratio.",
        Page::Progress => "Determinate and indeterminate progress, with reduced-motion support.",
        Page::Badge => "Compact labels with variants and optional icons.",
        Page::Card => "A surface with a title, description, content, action and footer.",
        Page::Alert => "Inline messages with accessible announcements.",
        Page::Separator => "Horizontal and vertical dividers, with optional centered labels.",
        Page::Collapsible => "Show and hide content with a button, Enter or Space.",
        Page::Button => "Actions with variants, icons, loading and accessible activation.",
        Page::Input => "Controlled single-line and search fields.",
        Page::TextArea => "Multiline editing, scrolling, clipping and resize capture.",
        Page::Checkbox => "Unchecked, checked and mixed states with keyboard and touch activation.",
        Page::Switch => "Animated binary preferences.",
        Page::RadioGroup => "Exclusive selection with semantic grouping.",
        Page::Slider => "Pointer, touch, keyboard and accessibility values.",
        Page::Tabs => "Roving navigation and one mounted panel.",
        Page::Select => "Anchored, collision-aware option overlay.",
        Page::Dialog => "Modal focus containment and restoration.",
        Page::Layout => "CSS-shaped Block, Flex, Grid, box model and text alignment.",
        Page::Motion => "Frame-paced feedback and interaction transitions.",
        Page::Effects => "Custom WGSL through the generic effect registry.",
        Page::Typography => "Rich spans, decoration, clamping and web-like text selection.",
        Page::WebView => "Retained web content, with separate email and webpage security policies.",
        Page::AsyncTasks => "Owned, cancellable work with event-driven delivery to the UI.",
        Page::Actions => "One command for buttons, menus, palettes and focused shortcuts.",
        Page::Editing => {
            "Transactional undo/redo, Unicode, filtered fields and protected passwords."
        }
        Page::CustomTimeline => {
            "Custom measurement and painting with draggable clips and standard controls."
        }
    }
}
