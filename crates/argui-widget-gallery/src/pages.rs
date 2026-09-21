use argui::{
    paint::{Border, CornerRadii},
    runtime::Entity,
    text::TextAlign,
    ui::{
        AlignItems, CursorIcon, Element, EventListener, GestureCapture, GestureSet, Interaction,
        JustifyContent, PanGesture, Position, Sides, auto, evenly_sized_tracks, length, percent,
    },
    widgets::{
        Button, Checkbox, Dialog, DialogBehavior, RadioGroup, RadioOption, RangeConfig, Select,
        SelectOption, Slider, Switch, Tab, TablerIcon, Tabs, TextArea, WidgetAssets, WidgetTheme,
    },
};

use crate::{
    app::{WidgetGallery, text},
    navigation::Page,
};

mod alert;
pub(crate) mod animated_text;
mod aspect_ratio;
pub(crate) mod async_tasks;
mod avatar;
mod badge;
mod breadcrumb;
mod buttons;
mod card;
pub(crate) mod catalogue;
mod collapsible;
pub(crate) mod color_picker;
pub(crate) mod damage_control;
pub(crate) mod data;
pub(crate) mod data_table;
pub(crate) mod dates;
pub(crate) mod drag_drop;
pub(crate) mod editing;
mod empty;
pub(crate) mod file_picker;
pub(crate) mod i18n;
mod inputs;
mod kbd;
mod label;
pub(crate) mod liquid_glass;
pub(crate) mod menus;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) mod mobile_activity;
pub(crate) mod motion;
pub(crate) mod overlay_effects;
mod pagination;
pub(crate) mod popover;
pub(crate) mod progress;
pub(crate) mod scroll_effects;
mod separator;
pub(crate) mod skeleton;
pub(crate) mod split_pane;
pub(crate) mod text_selection;
pub(crate) mod timeline;
pub(crate) mod toast;
pub(crate) mod tooltip;
mod typography;
#[cfg(feature = "updater")]
pub(crate) mod updater;
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
        Page::Accordion
        | Page::AlertDialog
        | Page::Attachment
        | Page::Bubble
        | Page::ButtonGroup
        | Page::Carousel
        | Page::Chart
        | Page::Combobox
        | Page::Direction
        | Page::Drawer
        | Page::Field
        | Page::HoverCard
        | Page::InputGroup
        | Page::InputOtp
        | Page::Item
        | Page::Marker
        | Page::Message
        | Page::MessageScroller
        | Page::NativeSelect
        | Page::NavigationMenu
        | Page::Questionnaire
        | Page::ScrollArea
        | Page::Sheet
        | Page::Sidebar
        | Page::Toggle
        | Page::ToggleGroup => cx.entity(&gallery.catalogue[&gallery.page]),
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
        Page::FilePicker => {
            let picker = gallery
                .file_picker
                .get_or_init(|| Entity::new(file_picker::FilePickerDemo::default()));
            cx.entity(picker)
        }
        Page::Empty => empty::render(theme, assets, cx),
        Page::Kbd => kbd::render(theme),
        Page::AspectRatio => aspect_ratio::render(theme, gallery.logo),
        Page::Progress => cx.entity(
            gallery
                .progress
                .get_or_init(|| argui::runtime::Entity::new(progress::ProgressDemo::default())),
        ),
        Page::Label => label::render(gallery, theme, cx),
        Page::Breadcrumb => breadcrumb::render(theme, cx),
        Page::Pagination => pagination::render(gallery, theme, cx),
        Page::Skeleton => cx.entity(
            gallery
                .skeleton
                .get_or_init(|| argui::runtime::Entity::new(skeleton::SkeletonDemo::default())),
        ),
        Page::Badge => badge::render(theme, assets),
        Page::Card => card::render(theme, assets, cx),
        Page::Alert => alert::render(theme, assets),
        Page::Separator => separator::render(theme),
        Page::Collapsible => collapsible::render(gallery, theme, assets, cx),
        Page::Button => buttons::render(gallery, theme, cx.entity(&gallery.spinner), cx),
        Page::Input => inputs::render(gallery, theme, assets, cx),
        Page::TextArea => textarea(
            gallery,
            theme,
            assets,
            cx,
            resize.textarea,
            resize.textarea_reset,
        ),
        Page::Checkbox => checkboxes(gallery, theme, assets, cx),
        Page::Switch => switches(gallery, theme, cx),
        Page::RadioGroup => radios(gallery, theme, cx),
        Page::AnimatedText => cx.entity(
            gallery
                .animated_text
                .get_or_init(|| Entity::new(animated_text::AnimatedTextDemo::default())),
        ),
        Page::ColorPicker => cx.entity(
            gallery
                .color_picker
                .get_or_init(|| Entity::new(color_picker::ColorPickerDemo::default())),
        ),
        Page::Slider => sliders(gallery, theme, assets, cx),
        Page::Tabs => tabs(gallery, theme, cx),
        Page::Select => selects(gallery, theme, assets),
        Page::Dialog => dialogs(gallery, theme, cx),
        Page::Popover => cx.entity(&gallery.popover),
        Page::Tooltip => cx.entity(&gallery.tooltip),
        Page::I18n => cx.entity(&gallery.i18n),
        Page::Layout => layout_system(theme),
        Page::Motion => cx.entity(&gallery.motion),
        Page::DamageControl => cx.entity(&gallery.damage_control),
        Page::LiquidGlass => cx.entity(&gallery.glass),
        Page::ScrollShadow => cx.entity(&gallery.scroll_demo),
        Page::Typography => typography::render(theme),
        Page::TextSelection => cx.entity(&gallery.text_selection),
        Page::WebView => cx.entity(&gallery.webview),
        Page::AsyncTasks => cx.entity(&gallery.tasks),
        #[cfg(any(target_os = "android", target_os = "ios"))]
        Page::MobileActivity => cx.entity(&gallery.mobile_activity),
        Page::Editing => cx.entity(&gallery.editing),
        Page::DragDrop => cx.entity(&gallery.drag_drop),
        Page::SplitPane => cx.entity(&gallery.split_pane),
        Page::CustomTimeline => cx.entity(&gallery.timeline),
        #[cfg(feature = "updater")]
        Page::Updater => cx.entity(
            gallery
                .updater
                .get_or_init(|| Entity::new(updater::UpdaterDemo::default())),
        ),
    };
    Element::column([
        Element::column([
            text(gallery.page.label(), 30.0, theme.foreground, 740),
            text(
                gallery.page.description(),
                15.0,
                theme.muted_foreground,
                400,
            ),
        ])
        .gap(5.0),
        content,
    ])
    .width(percent(1.0))
    .max_width(
        if matches!(gallery.page, Page::LiquidGlass | Page::ScrollShadow) {
            percent(1.0)
        } else {
            length(920.0)
        },
    )
    .gap(24.0)
}

fn textarea(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut argui::runtime::Context<WidgetGallery>,
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
        .on_input(cx.input_callback(|gallery, value| gallery.notes = value))
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

fn checkboxes(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut argui::runtime::Context<WidgetGallery>,
) -> Element {
    preview(
        "Team notifications",
        "Choose which project updates you receive. Space toggles the focused option.",
        Element::column([
            Checkbox::new(
                "accepted",
                "Notify me when a review is requested",
                if gallery.accepted {
                    argui::ui::CheckedState::Checked
                } else {
                    argui::ui::CheckedState::Unchecked
                },
            )
            .indicator(assets.icon(TablerIcon::Check, 14.0))
            .on_change(
                cx.value_callback(|gallery, value: argui::ui::CheckedState| {
                    gallery.accepted = value != argui::ui::CheckedState::Unchecked;
                }),
            )
            .build(theme),
            Checkbox::new(
                "check-empty",
                "Send a weekly email digest",
                if gallery.email_updates {
                    argui::ui::CheckedState::Checked
                } else {
                    argui::ui::CheckedState::Unchecked
                },
            )
            .on_change(
                cx.value_callback(|gallery, value: argui::ui::CheckedState| {
                    gallery.email_updates = value != argui::ui::CheckedState::Unchecked;
                }),
            )
            .build(theme),
            Checkbox::new(
                "check-disabled",
                "Security alerts (required)",
                argui::ui::CheckedState::Checked,
            )
            .enabled(false)
            .build(theme),
        ])
        .gap(8.0),
        theme,
    )
}

fn switches(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    cx: &mut argui::runtime::Context<WidgetGallery>,
) -> Element {
    preview(
        "Rendering preferences",
        "Enable profiling while you work, or prepare a render without a network connection.",
        Element::column([
            Switch::new("notifications", "GPU profiling", gallery.notifications)
                .on_change(cx.value_callback(|gallery, value| gallery.notifications = value))
                .build(theme),
            Switch::new("switch-off", "Offline rendering", gallery.offline_rendering)
                .on_change(cx.value_callback(|gallery, value| gallery.offline_rendering = value))
                .build(theme),
            Switch::new(
                "switch-disabled",
                "Cloud rendering (connection required)",
                true,
            )
            .enabled(false)
            .build(theme),
        ])
        .gap(8.0),
        theme,
    )
}

fn radios(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    cx: &mut argui::runtime::Context<WidgetGallery>,
) -> Element {
    preview(
        "Quality preset",
        "Choose a rendering preset. Use the arrow keys to compare the options.",
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
        .on_select(cx.value_callback(|gallery, selection| gallery.radio = selection))
        .build(theme),
        theme,
    )
}

fn sliders(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut argui::runtime::Context<WidgetGallery>,
) -> Element {
    preview(
        "Continuous and stepped input",
        "Drag a slider or use the arrow keys to adjust the value. Home and End reach the limits.",
        Element::column([
            crate::property_slider::render(gallery, theme, assets),
            Slider::new(
                "plain-slider",
                "Plain slider",
                gallery.plain_slider,
                RangeConfig::default(),
            )
            .on_change(cx.value_callback(|gallery, value| gallery.plain_slider = value))
            .on_commit(cx.value_callback(|gallery, value| gallery.plain_slider = value))
            .build(theme),
        ])
        .gap(12.0),
        theme,
    )
}

fn tabs(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    cx: &mut argui::runtime::Context<WidgetGallery>,
) -> Element {
    preview(
        "Tabbed settings",
        "Switch categories to explore the available workspace settings.",
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
        .on_select(cx.value_callback(|gallery, selection| gallery.tab = selection))
        .build(theme),
        theme,
    )
}

fn selects(gallery: &WidgetGallery, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    preview(
        "Backend selection",
        "Choose the backend for your next render. Type a name to jump to an option.",
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

fn dialogs(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    cx: &mut argui::runtime::Context<WidgetGallery>,
) -> Element {
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
        .on_open_change(cx.value_callback(|gallery, open| gallery.dialog_open = open))
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
