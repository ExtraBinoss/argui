use argui::{
    core::{Color, Key, KeyState, Size},
    paint::{Border, BorderWidths, CornerRadii, ImageFit, ImageId},
    runtime::{Context, Entity, LayoutSnapshot, WindowEnvironment},
    text::{TextColor, TextStyle, TextWrap},
    theme::ThemeMode,
    ui::{
        AlignItems, Axes, CursorIcon, Element, GestureSet, Interaction, JustifyContent,
        KeyboardActivation, Overflow, Role, ScrollConfig, ScrollRequest, SemanticAction, Semantics,
        Sides, UiEvent, UiEventKind, length, percent,
    },
    widgets::{
        Button, DialogAction, DialogBehavior, RadioGroupAction, RadioGroupBehavior, RangeBehavior,
        RangeConfig, RangeState, SelectAction, SelectBehavior, SelectOption, Spinner, TablerIcon,
        TabsAction, TabsBehavior, WidgetAssets, WidgetTheme,
    },
};
use argui_image::ImageLibrary;

use crate::{navigation::Page, pages};

mod desktop_backdrop;
mod interaction;
mod navigation;
mod theme;
pub(crate) use theme::PRIMARIES;
use theme::mode_label;

const EDITOR_DEFAULT_SIZE: Size = Size::new(520.0, 170.0);

pub struct WidgetGallery {
    backdrop: desktop_backdrop::BackdropSettings,
    pub(crate) page: Page,
    pub(crate) search: String,
    search_highlight: usize,
    navigation_root: Element,
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
    pub(crate) tasks: Entity<pages::async_tasks::TasksDemo>,
    pub(crate) actions: Entity<pages::actions::ActionsDemo>,
    pub(crate) editing: Entity<pages::editing::EditingDemo>,
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
            backdrop: desktop_backdrop::BackdropSettings::default(),
            scroll_demo: Entity::new(pages::scroll_effects::ScrollDemo::default()),
            webview: pages::webview::WebViewDemo::entity(),
            glass: Entity::new(pages::liquid_glass::GlassDemo::default()),
            menus: Entity::new(pages::menus::MenusDemo::new(&dark_assets)),
            dates: Entity::new(pages::dates::DatesDemo::new(&dark_assets)),
            data_table: std::cell::OnceCell::new(),
            popover: Entity::new(pages::popover::PopoverDemo::default()),
            tooltip: Entity::new(pages::tooltip::TooltipDemo::default()),
            toasts: Entity::new(pages::toast::ToastDemo::new(&dark_assets)),
            data: Entity::new(pages::data::DataDemo::default()),
            tasks: Entity::new(pages::async_tasks::TasksDemo::default()),
            actions: Entity::new(pages::actions::ActionsDemo::default()),
            editing: Entity::new(pages::editing::EditingDemo::default()),
            timeline: Entity::new(pages::timeline::Timeline::default()),
            page: Page::Button,
            search: String::new(),
            search_highlight: 0,
            navigation_root: Element::container([]),
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
    fn view(
        &self,
        environment: WindowEnvironment,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
        resize: pages::ResizeListeners,
    ) -> Element {
        Element::column([
            self.topbar(theme, assets, environment),
            Element::row([
                self.sidebar(theme, assets),
                Element::container([pages::render(self, theme, assets, cx, resize)])
                    .keyed("gallery-content-scroll")
                    .background(theme.background)
                    .grow(1.0)
                    .min_width(length(0.0))
                    .min_height(length(0.0))
                    .padding(Sides::length(30.0))
                    .overflow(Axes {
                        x: Overflow::Hidden,
                        y: Overflow::Auto,
                    })
                    .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone())),
            ])
            .grow(1.0)
            .min_height(length(0.0)),
            cx.entity(&self.toasts),
        ])
        .keyed("gallery-root")
        .focus_scope(argui::ui::FocusScope {
            initial: Some(argui::ui::InitialFocus::Target("gallery-root".into())),
            ..argui::ui::FocusScope::restoring().restore(false)
        })
        .width(percent(1.0))
        .height(percent(1.0))
        .background(Color::TRANSPARENT)
        .interaction(
            Interaction::default()
                .focus_policy(argui::ui::FocusPolicy::TabStop)
                .gestures(GestureSet::default().tap(argui::ui::TapGesture::default())),
        )
        .semantics(
            Semantics::new(Role::Window)
                .label("Argui Widget Gallery")
                .description(format!(
                    "{} theme, {} page",
                    mode_label(self.theme_mode),
                    self.page.label()
                )),
        )
        .inspectable(true)
    }

    fn topbar(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        environment: WindowEnvironment,
    ) -> Element {
        let mode_icon = match self.theme_mode {
            ThemeMode::Light => TablerIcon::Sun,
            ThemeMode::Dark => TablerIcon::Moon,
            ThemeMode::System => TablerIcon::System,
        };
        let brand = Element::row([
            Element::image(self.logo)
                .image_fit(ImageFit::Contain)
                .width(length(34.0))
                .height(length(34.0))
                .semantics(Semantics::new(Role::Image).label("Argui Astra logo")),
            Element::column([
                text("ARGUI", 16.0, theme.foreground, 750),
                text("Widget Gallery", 12.0, theme.muted_foreground, 500),
            ])
            .gap(1.0),
        ])
        .align_items(AlignItems::CENTER)
        .gap(10.0);
        let swatches = Element::row(PRIMARIES.iter().enumerate().map(|(index, color)| {
            Element::container([])
                .keyed(format!("primary::{index}"))
                .width(length(if self.primary == index { 20.0 } else { 16.0 }))
                .height(length(if self.primary == index { 20.0 } else { 16.0 }))
                .background(*color)
                .border(Border::all(
                    if self.primary == index { 2.0 } else { 1.0 },
                    if self.primary == index {
                        theme.foreground
                    } else {
                        theme.border
                    },
                ))
                .radius(CornerRadii::all(999.0))
                .interaction(
                    Interaction::default()
                        .focus_policy(argui::ui::FocusPolicy::TabStop)
                        .cursor(CursorIcon::Pointer)
                        .gestures(GestureSet::default().tap(argui::ui::TapGesture::default()))
                        .keyboard_activation(KeyboardActivation::EnterOrSpace),
                )
                .semantics(
                    Semantics::new(Role::Button)
                        .label(format!("Primary color {}", index + 1))
                        .action(SemanticAction::Click),
                )
        }))
        .gap(8.0)
        .align_items(AlignItems::CENTER);
        let theme_button = Button::new(
            "theme-mode",
            mode_label(self.theme_mode),
            theme.ghost_button(),
        )
        .leading(assets.icon(mode_icon, 17.0))
        .build();
        let mut controls = vec![swatches];
        if cfg!(feature = "desktop-backdrop") {
            controls.push(self.backdrop_controls(theme, environment));
        }
        controls.push(theme_button);
        Element::row([brand, Element::row(controls).gap(10.0)])
            .height(length(64.0))
            .padding(Sides {
                left: length(22.0),
                right: length(126.0),
                top: length(12.0),
                bottom: length(12.0),
            })
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .background(theme.card)
            .border(Border {
                widths: BorderWidths {
                    bottom: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
    }

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
