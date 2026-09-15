use argui::{
    core::{Color, Key, KeyState, Size},
    paint::ImageId,
    runtime::{Context, Entity, LayoutSnapshot},
    text::{TextColor, TextStyle, TextWrap},
    theme::ThemeMode,
    ui::{Element, ScrollRequest, UiEvent, UiEventKind},
    widgets::{
        DialogAction, DialogBehavior, RadioGroupAction, RadioGroupBehavior, RangeBehavior,
        RangeConfig, RangeState, SelectAction, SelectBehavior, SelectOption, Spinner, TablerIcon,
        TabsAction, TabsBehavior, WidgetAssets,
    },
};
use argui_image::ImageLibrary;

use crate::{navigation::Page, pages};

mod desktop_backdrop;
mod interaction;
mod layout;
mod navigation;
mod theme;
pub(crate) use theme::PRIMARIES;
use theme::mode_label;

const EDITOR_DEFAULT_SIZE: Size = Size::new(520.0, 170.0);

/// Interactive gallery demonstrating Argui widgets and their behaviors.
///
/// # Panics
///
/// [`Default::default`] panics if the bundled gallery icon is not valid PNG data.
pub struct WidgetGallery {
    pub(crate) catalogue: std::collections::HashMap<Page, Entity<pages::catalogue::CatalogueDemo>>,
    backdrop: desktop_backdrop::BackdropSettings,
    pub(crate) page: Page,
    pub(crate) search: String,
    search_highlight: usize,
    navigation_root: Element,
    compact: bool,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) primary: usize,
    pub(crate) name: String,
    pub(crate) workspace_id: String,
    pub(crate) email: String,
    pub(crate) input_search: String,
    pub(crate) invalid_email: String,
    pub(crate) notes: String,
    pub(crate) accepted: bool,
    pub(crate) notifications: bool,
    pub(crate) email_updates: bool,
    pub(crate) offline_rendering: bool,
    pub(crate) radio: usize,
    pub(crate) slider: f32,
    pub(crate) plain_slider: f32,
    pub(crate) slider_editing: bool,
    pub(crate) slider_edit_value: String,
    pub(crate) tab: usize,
    pub(crate) select_presence: argui::widgets::Presence,
    select_search: argui::widgets::Typeahead,
    search_clock: web_time::Instant,
    pub(crate) select_highlight: usize,
    pub(crate) select_selected: Option<usize>,
    pub(crate) dialog_open: bool,
    pub(crate) collapsible_open: bool,
    pub(crate) archived_open: bool,
    pub(crate) clicks: u32,
    pub(crate) result_page: usize,
    pub(crate) skeleton: std::cell::OnceCell<Entity<pages::skeleton::SkeletonDemo>>,
    pub(crate) editor_size: Size,
    editor_resize_start: Size,
    pub(crate) scroll_demo: Entity<pages::scroll_effects::ScrollDemo>,
    pub(crate) webview: Entity<pages::webview::WebViewDemo>,
    pub(crate) glass: Entity<pages::liquid_glass::GlassDemo>,
    pub(crate) menus: Entity<pages::menus::MenusDemo>,
    pub(crate) dates: Entity<pages::dates::DatesDemo>,
    pub(crate) data_table: std::cell::OnceCell<Entity<pages::data_table::TableDemo>>,
    pub(crate) popover: Entity<pages::popover::PopoverDemo>,
    pub(crate) tooltip: Entity<pages::tooltip::TooltipDemo>,
    pub(crate) toasts: Entity<pages::toast::ToastDemo>,
    pub(crate) data: Entity<pages::data::DataDemo>,
    #[cfg(feature = "updater")]
    pub(crate) updater: std::cell::OnceCell<Entity<pages::updater::UpdaterDemo>>,
    pub(crate) tasks: Entity<pages::async_tasks::TasksDemo>,
    pub(crate) motion: Entity<pages::motion::MotionDemo>,
    pub(crate) text_selection: Entity<pages::text_selection::SelectionDemo>,
    pub(crate) editing: Entity<pages::editing::EditingDemo>,
    pub(crate) hot_reload: Entity<pages::hot_reload::HotReloadDemo>,
    pub(crate) i18n: Entity<pages::i18n::I18nDemo>,
    pub(crate) timeline: Entity<pages::timeline::Timeline>,
    pub(crate) slider_state: RangeState,
    pub(crate) plain_slider_state: RangeState,
    images: ImageLibrary,
    pub(crate) logo: ImageId,
    light_assets: WidgetAssets,
    dark_assets: WidgetAssets,
    accent_assets: WidgetAssets,
    pub(crate) spinner: Entity<Spinner>,
    pub(crate) file_picker: std::cell::OnceCell<Entity<pages::file_picker::FilePickerDemo>>,
    pub(crate) progress: std::cell::OnceCell<Entity<pages::progress::ProgressDemo>>,
    pub(crate) animated_text: std::cell::OnceCell<Entity<pages::animated_text::AnimatedTextDemo>>,
    pub(crate) color_picker: std::cell::OnceCell<Entity<pages::color_picker::ColorPickerDemo>>,
}

impl Default for WidgetGallery {
    fn default() -> Self {
        let mut images = ImageLibrary::new();
        let logo = images
            .insert(include_bytes!(
                "../../argui/examples/assets/astra-icon-256.png"
            ))
            .expect("the embedded Astra icon must remain a valid PNG");
        let light_assets = WidgetAssets::tabler(Color::srgb(0.18, 0.20, 0.25));
        let dark_assets = WidgetAssets::tabler(Color::srgb(0.88, 0.90, 0.95));
        let accent_assets = WidgetAssets::tabler(PRIMARIES[0]);
        let spinner = Entity::new(Spinner::new(
            accent_assets.vector_id(TablerIcon::Loader),
            17.0,
        ));
        Self {
            catalogue: pages::catalogue::PAGES
                .into_iter()
                .map(|page| {
                    (
                        page,
                        Entity::new(pages::catalogue::CatalogueDemo::new(page)),
                    )
                })
                .collect(),
            backdrop: desktop_backdrop::BackdropSettings::default(),
            scroll_demo: Entity::new(pages::scroll_effects::ScrollDemo::default()),
            webview: pages::webview::WebViewDemo::entity(),
            glass: Entity::new(pages::liquid_glass::GlassDemo::new(&light_assets)),
            menus: Entity::new(pages::menus::MenusDemo::new(&dark_assets)),
            dates: Entity::new(pages::dates::DatesDemo::new(&dark_assets)),
            data_table: std::cell::OnceCell::new(),
            color_picker: std::cell::OnceCell::new(),
            animated_text: std::cell::OnceCell::new(),
            popover: Entity::new(pages::popover::PopoverDemo::default()),
            tooltip: Entity::new(pages::tooltip::TooltipDemo::default()),
            toasts: Entity::new(pages::toast::ToastDemo::new(&dark_assets)),
            data: Entity::new(pages::data::DataDemo::default()),
            #[cfg(feature = "updater")]
            updater: std::cell::OnceCell::new(),
            tasks: Entity::new(pages::async_tasks::TasksDemo::default()),
            motion: Entity::new(pages::motion::MotionDemo::default()),
            text_selection: Entity::new(pages::text_selection::SelectionDemo::default()),
            editing: Entity::new(pages::editing::EditingDemo::default()),
            hot_reload: Entity::new(pages::hot_reload::HotReloadDemo::default()),
            i18n: Entity::new(pages::i18n::I18nDemo::default()),
            timeline: Entity::new(pages::timeline::Timeline::default()),
            page: Page::Button,
            search: String::new(),
            search_highlight: 0,
            navigation_root: Element::container([]),
            compact: false,
            theme_mode: ThemeMode::System,
            primary: 0,
            name: "Ada Lovelace".into(),
            workspace_id: "argui-studio".into(),
            email: "ada@example.com".into(),
            input_search: String::new(),
            invalid_email: "broken@".into(),
            notes: "Argui widgets stay controlled by your application state.".into(),
            accepted: true,
            notifications: true,
            email_updates: false,
            offline_rendering: false,
            radio: 0,
            slider: 64.0,
            plain_slider: 42.0,
            slider_editing: false,
            slider_edit_value: "64".into(),
            tab: 0,
            select_presence: argui::widgets::Presence::default().fade_in(false),
            select_search: argui::widgets::Typeahead::default(),
            search_clock: web_time::Instant::now(),
            select_highlight: 0,
            select_selected: Some(0),
            dialog_open: false,
            collapsible_open: false,
            archived_open: false,
            clicks: 0,
            result_page: 1,
            skeleton: std::cell::OnceCell::new(),
            editor_size: EDITOR_DEFAULT_SIZE,
            editor_resize_start: EDITOR_DEFAULT_SIZE,
            slider_state: RangeState::default(),
            plain_slider_state: RangeState::default(),
            images,
            logo,
            light_assets,
            dark_assets,
            accent_assets,
            spinner,
            file_picker: std::cell::OnceCell::new(),
            progress: std::cell::OnceCell::new(),
        }
    }
}

impl WidgetGallery {
    fn select_options() -> Vec<SelectOption> {
        ["Vulkan", "DirectX 12", "Metal", "WebGPU"]
            .into_iter()
            .map(SelectOption::new)
            .collect()
    }

    fn handle_shortcuts(&mut self, event: &UiEvent, cx: &mut Context<Self>) -> bool {
        let UiEventKind::KeyInput(input) = &event.kind else {
            return false;
        };
        if input.state != KeyState::Pressed || !input.modifiers.command() {
            return false;
        }
        match &input.key {
            Key::Character(value) if value.eq_ignore_ascii_case("k") => {
                cx.request_focus("gallery-search");
                true
            }
            Key::Character(value) if input.modifiers.shift && value.eq_ignore_ascii_case("l") => {
                self.cycle_theme();
                cx.set_theme(self.theme_request());
                true
            }
            _ => false,
        }
    }
}

impl WidgetGallery {
    fn handle_event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if self.backdrop_event(event, cx) {
            return;
        }
        if self.page == Page::Popover {
            let dismissed = self.popover.update(|demo, cx| {
                let dismissed = demo.dismiss(event);
                if dismissed {
                    cx.notify();
                }
                dismissed
            });
            if dismissed {
                event.stop_propagation();
                return;
            }
        }
        if self.page == Page::Tooltip
            && matches!(&event.kind, UiEventKind::KeyInput(input) if input.key == Key::Escape)
        {
            self.tooltip.update(|demo, cx| {
                if demo.dismiss(event) {
                    cx.notify();
                }
            });
        }
        if self.handle_shortcuts(event, cx) {
            event.stop_propagation();
            return;
        }
        if self.type_to_search(event, cx) {
            return;
        }
        if let Some(page) = self.update_search_keys(event) {
            self.select_page(page);
            cx.notify();
            return;
        }
        if crate::property_slider::update(self, event, cx) {
            return;
        }
        let plain_slider = RangeBehavior::new(
            "plain-slider",
            "Plain slider",
            self.plain_slider,
            RangeConfig::default(),
        );
        if let Some(action) = self.plain_slider_state.update(event, &plain_slider) {
            self.plain_slider = action.value();
            cx.notify();
            return;
        }
        if let UiEventKind::TextChanged(value) = &event.kind {
            match event.target_key() {
                Some("gallery-search") => {
                    self.search.clone_from(value);
                    self.search_highlight = 0;
                }
                Some("name") => self.name.clone_from(value),
                Some("workspace-id") => self.workspace_id.clone_from(value),
                Some("email") => self.email.clone_from(value),
                Some("input-search-demo") => self.input_search.clone_from(value),
                Some("invalid") => self.invalid_email.clone_from(value),
                Some("notes") => self.notes.clone_from(value),
                _ => return,
            }
            cx.notify();
            return;
        }
        if let Some(page) = event
            .target_key()
            .and_then(Page::from_navigation_key)
            .filter(|_| matches!(event.kind, UiEventKind::Click(_)))
        {
            self.select_page(page);
            cx.notify();
            return;
        }
        if matches!(event.kind, UiEventKind::Click(_)) {
            match event.target_key() {
                Some("theme-mode") => {
                    self.cycle_theme();
                    cx.set_theme(self.theme_request());
                    return;
                }
                Some(
                    "demo-button" | "secondary" | "outline" | "ghost" | "danger"
                    | "button-elevated" | "button-lift" | "button-shader",
                ) => {
                    self.clicks = self.clicks.saturating_add(1);
                    cx.notify();
                    return;
                }
                Some("accepted") => {
                    self.accepted = !self.accepted;
                    cx.notify();
                    return;
                }
                Some("check-empty" | "switch-off") => {
                    let value = if event.target_key() == Some("check-empty") {
                        &mut self.email_updates
                    } else {
                        &mut self.offline_rendering
                    };
                    *value = !*value;
                    cx.notify();
                    return;
                }
                Some("notifications") => {
                    self.notifications = !self.notifications;
                    cx.notify();
                    return;
                }
                _ => {
                    if let Some(index) = event
                        .target_key()
                        .and_then(|key| key.strip_prefix("primary::"))
                        .and_then(|value| value.parse::<usize>().ok())
                        .filter(|index| *index < PRIMARIES.len())
                    {
                        self.primary = index;
                        cx.set_theme(self.theme_request());
                        return;
                    }
                }
            }
        }
        let radio = RadioGroupBehavior::new(
            "quality",
            "Quality",
            [
                ("Maximum quality".into(), true),
                ("Balanced".into(), true),
                ("Performance".into(), true),
            ],
            Some(self.radio),
        );
        if let Some(RadioGroupAction::Select(selection)) = radio.action(event) {
            self.radio = selection;
            cx.notify();
            return;
        }
        let tabs = TabsBehavior::new(
            "demo-tabs",
            [
                ("General".into(), true),
                ("Performance".into(), true),
                ("Advanced".into(), true),
            ],
            self.tab,
        );
        if let Some(TabsAction::Select(selection)) = tabs.action(event) {
            self.tab = selection;
            cx.notify();
            return;
        }
        let options = Self::select_options();
        let select =
            SelectBehavior::new("backend", "Choose a backend", options, self.select_selected)
                .open(self.select_presence.is_open())
                .highlighted(self.select_highlight);
        if let Some(action) = select
            .search(event, &mut self.select_search, self.search_clock.elapsed())
            .or_else(|| select.action(event))
        {
            match action {
                SelectAction::Toggle => self.select_presence.set_open(
                    !self.select_presence.is_open(),
                    cx.environment().reduced_motion,
                ),
                SelectAction::Close => self
                    .select_presence
                    .set_open(false, cx.environment().reduced_motion),
                SelectAction::Highlight(index) => {
                    self.select_highlight = index;
                    cx.request_focus(select.option_key(index));
                    cx.scroll(ScrollRequest::reveal(select.option_key(index)));
                }
                SelectAction::Select(index) => {
                    self.select_selected = Some(index);
                    self.select_highlight = index;
                    self.select_presence
                        .set_open(false, cx.environment().reduced_motion);
                }
            }
            cx.notify();
            return;
        }
        if let Some(action) =
            DialogBehavior::new("demo-dialog", "Delete GPU cache", self.dialog_open).action(event)
        {
            self.dialog_open = action == DialogAction::Open;
            cx.notify();
        }
    }

    fn handle_layout(&mut self, layout: &LayoutSnapshot) {
        self.backdrop.layout_changed(layout);
        crate::property_slider::layout_changed(self, layout);
        let behavior = RangeBehavior::new(
            "plain-slider",
            "Plain slider",
            self.plain_slider,
            RangeConfig::default(),
        );
        self.plain_slider_state.layout_changed(layout, &behavior);
    }
}

pub(crate) fn text(value: impl Into<String>, size: f32, color: TextColor, weight: u16) -> Element {
    Element::text(value.into()).text_style(TextStyle {
        font_size: size,
        line_height: size * 1.35,
        color,
        weight,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    })
}
