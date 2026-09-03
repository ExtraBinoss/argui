use argui::{
    core::{Color, ColorScheme, Key, KeyState, Point, Size},
    paint::{Border, BorderWidths, CornerRadii, ImageAsset, ImageFit, ImageId, VectorAsset},
    runtime::{Context, Entity, LayoutSnapshot, Render, ThemeRequest, WindowEnvironment},
    text::{TextColor, TextStyle, TextWrap},
    theme::ThemeMode,
    ui::{
        AlignItems, Axes, CursorIcon, Element, GestureSet, Interaction, JustifyContent,
        KeyboardActivation, Overflow, ResizeConfig, ResizeState, Role, ScrollConfig,
        SemanticAction, Semantics, Sides, UiEvent, UiEventKind, length, percent, sides,
    },
    widgets::{
        Button, DialogAction, DialogBehavior, Input, InputKind, RadioGroupAction,
        RadioGroupBehavior, RangeBehavior, RangeConfig, RangeState, SelectAction, SelectBehavior,
        SelectOption, Spinner, TablerIcon, TabsAction, TabsBehavior, WidgetAssets, WidgetTheme,
        shadcn,
    },
};
use argui_image::ImageLibrary;

use crate::{navigation::Page, pages};

pub(crate) static PRIMARIES: LazyLock<[Color; 6]> = LazyLock::new(|| {
    [
        Color::srgb(0.10, 0.45, 0.91),
        Color::srgb(0.49, 0.23, 0.93),
        Color::srgb(0.86, 0.20, 0.45),
        Color::srgb(0.04, 0.62, 0.48),
        Color::srgb(0.92, 0.42, 0.08),
        Color::srgb(0.20, 0.68, 0.94),
    ]
});

pub struct WidgetGallery {
    pub(crate) page: Page,
    pub(crate) search: String,
    search_highlight: usize,
    pub(crate) theme_mode: ThemeMode,
    pub(crate) primary: usize,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) notes: String,
    pub(crate) accepted: bool,
    pub(crate) notifications: bool,
    pub(crate) radio: usize,
    pub(crate) slider: f32,
    pub(crate) plain_slider: f32,
    pub(crate) slider_editing: bool,
    pub(crate) slider_edit_value: String,
    pub(crate) tab: usize,
    pub(crate) select_open: bool,
    pub(crate) select_highlight: usize,
    pub(crate) select_selected: Option<usize>,
    pub(crate) dialog_open: bool,
    pub(crate) clicks: u32,
    pub(crate) composition_hits: u32,
    pub(crate) editor_size: ResizeState,
    pub(crate) slider_state: RangeState,
    pub(crate) plain_slider_state: RangeState,
    images: ImageLibrary,
    logo: ImageId,
    light_assets: WidgetAssets,
    dark_assets: WidgetAssets,
    accent_assets: WidgetAssets,
    spinner: Entity<Spinner>,
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
            page: Page::Button,
            search: String::new(),
            search_highlight: 0,
            theme_mode: ThemeMode::System,
            primary: 0,
            name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            notes: "Argui widgets stay controlled by your application state.".into(),
            accepted: true,
            notifications: true,
            radio: 0,
            slider: 64.0,
            plain_slider: 42.0,
            slider_editing: false,
            slider_edit_value: "64".into(),
            tab: 0,
            select_open: false,
            select_highlight: 0,
            select_selected: Some(0),
            dialog_open: false,
            clicks: 0,
            composition_hits: 0,
            editor_size: ResizeState::new(Size::new(520.0, 170.0)),
            slider_state: RangeState::default(),
            plain_slider_state: RangeState::default(),
            images,
            logo,
            light_assets,
            dark_assets,
            accent_assets,
            spinner,
        }
    }
}

impl WidgetGallery {
    fn view(
        &self,
        _environment: WindowEnvironment,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        spinner: Element,
    ) -> Element {
        Element::column([
            self.topbar(theme, assets),
            Element::row([
                self.sidebar(theme, assets),
                pages::render(self, theme, assets, spinner)
                    .keyed("gallery-content-scroll")
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
        ])
        .keyed("gallery-root")
        .width(percent(1.0))
        .height(percent(1.0))
        .background(theme.background)
        .interaction(
            Interaction::default()
                .focusable(true)
                .gestures(GestureSet::NONE.tap()),
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

    fn topbar(&self, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
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
                        .focusable(true)
                        .cursor(CursorIcon::Pointer)
                        .gestures(GestureSet::NONE.tap())
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
            theme.ghost_button.clone(),
        )
        .leading(assets.icon(mode_icon, 17.0))
        .build();
        Element::row([brand, Element::row([swatches, theme_button]).gap(14.0)])
            .height(length(64.0))
            .padding(sides(22.0, 12.0))
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

    fn sidebar(&self, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
        let search = Input::new(
            "gallery-search",
            self.search.clone(),
            "Search components…",
            theme.input.clone(),
        )
        .kind(InputKind::Search)
        .label("Search components")
        .description("Ctrl or Command K")
        .leading(assets.icon(TablerIcon::Search, 16.0), 38.0)
        .build();
        let mut children = vec![search];
        for category in ["Widgets", "Examples"] {
            let pages = Page::ALL
                .into_iter()
                .filter(|page| page.category() == category && page.matches(&self.search))
                .collect::<Vec<_>>();
            if pages.is_empty() {
                continue;
            }
            children.push(text(category, 11.0, theme.muted_foreground, 700));
            children.extend(pages.into_iter().map(|page| {
                Button::new(
                    format!("nav::{}", page.slug()),
                    page.label(),
                    if page == self.page {
                        theme.secondary_button.clone()
                    } else {
                        theme.ghost_button.clone()
                    },
                )
                .build()
                .width(percent(1.0))
            }));
        }
        Element::column(children)
            .width(length(260.0))
            .height(percent(1.0))
            .padding(Sides::length(16.0))
            .gap(9.0)
            .background(theme.card)
            .border(Border {
                widths: BorderWidths {
                    right: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
    }

    fn theme_request(&self) -> ThemeRequest {
        ThemeRequest {
            color_scheme: match self.theme_mode {
                ThemeMode::Light => Some(ColorScheme::Light),
                ThemeMode::Dark => Some(ColorScheme::Dark),
                ThemeMode::System => None,
            },
            primary: Some(PRIMARIES[self.primary]),
        }
    }

    fn cycle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
            ThemeMode::System => ThemeMode::Light,
        };
    }

    fn filtered_pages(&self) -> Vec<Page> {
        Page::ALL
            .into_iter()
            .filter(|page| page.matches(&self.search))
            .collect()
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

    fn update_search_keys(&mut self, event: &UiEvent) -> Option<Page> {
        if event.key.as_deref() != Some("gallery-search") {
            return None;
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed {
            return None;
        }
        let pages = self.filtered_pages();
        if pages.is_empty() {
            self.search_highlight = 0;
            return None;
        }
        match input.key {
            Key::ArrowDown => self.search_highlight = (self.search_highlight + 1) % pages.len(),
            Key::ArrowUp => {
                self.search_highlight = (self.search_highlight + pages.len() - 1) % pages.len();
            }
            Key::Home => self.search_highlight = 0,
            Key::End => self.search_highlight = pages.len() - 1,
            Key::Enter => return pages.get(self.search_highlight).copied(),
            _ => {}
        }
        None
    }
}

impl Render for WidgetGallery {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let themes = shadcn(environment.primary);
        let theme = themes.resolve(environment.color_scheme);
        let assets = match environment.color_scheme {
            ColorScheme::Light => &self.light_assets,
            ColorScheme::Dark => &self.dark_assets,
        };
        let spinner = cx.entity(&self.spinner);
        self.view(environment, theme, assets, spinner)
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if self.handle_shortcuts(event, cx) {
            event.stop_propagation();
            return;
        }
        if let Some(page) = self.update_search_keys(event) {
            self.page = page;
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
            match event.key.as_deref() {
                Some("gallery-search") => {
                    self.search.clone_from(value);
                    self.search_highlight = 0;
                }
                Some("name") => self.name.clone_from(value),
                Some("email") => self.email.clone_from(value),
                Some("notes") => self.notes.clone_from(value),
                _ => return,
            }
            cx.notify();
            return;
        }
        if let Some(page) = event
            .key
            .as_deref()
            .and_then(Page::from_navigation_key)
            .filter(|_| matches!(event.kind, UiEventKind::Clicked))
        {
            self.page = page;
            cx.notify();
            return;
        }
        if event.kind == UiEventKind::Clicked {
            match event.key.as_deref() {
                Some("theme-mode") => {
                    self.cycle_theme();
                    cx.set_theme(self.theme_request());
                    return;
                }
                Some("demo-button") => self.clicks = self.clicks.saturating_add(1),
                Some("composition-circle") => {
                    self.composition_hits = self.composition_hits.saturating_add(1);
                    cx.notify();
                    return;
                }
                Some("accepted") => self.accepted = !self.accepted,
                Some("notifications") => self.notifications = !self.notifications,
                _ => {
                    if let Some(index) = event
                        .key
                        .as_deref()
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
                .open(self.select_open)
                .highlighted(self.select_highlight);
        if let Some(action) = select.action(event) {
            match action {
                SelectAction::Toggle => self.select_open = !self.select_open,
                SelectAction::Close => self.select_open = false,
                SelectAction::Highlight(index) => {
                    self.select_highlight = index;
                    cx.request_focus(select.option_key(index));
                    cx.scroll_to(select.list_key(), Point::new(0.0, index as f32 * 36.0));
                }
                SelectAction::Select(index) => {
                    self.select_selected = Some(index);
                    self.select_highlight = index;
                    self.select_open = false;
                    cx.request_focus("backend");
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
            return;
        }
        if self
            .editor_size
            .update(
                event,
                "notes-resize",
                ResizeConfig::new(Size::new(280.0, 120.0), Size::new(760.0, 480.0)),
            )
            .is_some()
        {
            cx.notify();
        }
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, _cx: &mut Context<Self>) {
        crate::property_slider::layout_changed(self, layout);
        let behavior = RangeBehavior::new(
            "plain-slider",
            "Plain slider",
            self.plain_slider,
            RangeConfig::default(),
        );
        self.plain_slider_state.layout_changed(layout, &behavior);
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.images.assets().to_vec()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.light_assets
            .assets()
            .iter()
            .chain(self.dark_assets.assets())
            .chain(self.accent_assets.assets())
            .cloned()
            .collect()
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

const fn mode_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "Light",
        ThemeMode::Dark => "Dark",
        ThemeMode::System => "System",
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
use std::sync::LazyLock;
