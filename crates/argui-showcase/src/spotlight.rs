use argui_animation::{Duration, curves};
use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_paint::{Border, BorderWidths, CornerRadii, PaintStyle, QuadStyle, VectorAsset};
use argui_platform::{
    GlobalShortcutState, PlatformEvent, WindowBackend, WindowCapabilities, WindowKey,
};
use argui_runtime::{AppCommand, AppEvent, AppModel, AppUpdate, ViewUpdate, WindowEnvironment};
use argui_text::TextStyle;
use argui_ui::{
    AlignItems, Axes, DesktopBackdrop, Element, FocusContainment, FocusScope, FocusTarget,
    InitialFocus, JustifyContent, LiveRegion, Overflow, Role, Semantics, UiEventKind, length,
    percent, sides,
};
use argui_widgets::{AnimatedContainer, Input, InputKind, TablerIcon, WidgetAssets, shadcn};

use crate::spotlight_components::{compact_button, result_row, shortcut_hint, with_alpha};

const SEARCH_KEY: &str = "spotlight-search";
/// Stable ID used by the Spotlight example's global activation shortcut.
pub const SPOTLIGHT_SHORTCUT_ID: &str = "activate-spotlight";
const PANEL_WIDTH: f32 = 704.0;
const PANEL_HEIGHT: f32 = 452.0;
const PANEL_RADIUS: f32 = 18.0;
const RESULT_KEY_PREFIX: &str = "spotlight-result::";
const SPOTLIGHT_ICONS: [TablerIcon; 7] = [
    TablerIcon::Search,
    TablerIcon::Terminal,
    TablerIcon::Folder,
    TablerIcon::Sun,
    TablerIcon::Rust,
    TablerIcon::Minus,
    TablerIcon::ChevronDown,
];
/// Recommended width in logical pixels for the Spotlight showcase window.
pub const SPOTLIGHT_WINDOW_WIDTH: f64 = PANEL_WIDTH as f64;
/// Recommended height in logical pixels for the Spotlight showcase window.
pub const SPOTLIGHT_WINDOW_HEIGHT: f64 = PANEL_HEIGHT as f64;

#[derive(Clone, Copy)]
struct SpotlightResult {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    icon: TablerIcon,
    shortcut: &'static str,
}

const SPOTLIGHT_RESULTS: [SpotlightResult; 4] = [
    SpotlightResult {
        id: "command-palette",
        label: "Open command palette",
        description: "Jump to any action",
        icon: TablerIcon::Terminal,
        shortcut: "MOD K",
    },
    SpotlightResult {
        id: "recent-files",
        label: "Browse recent files",
        description: "Files and folders",
        icon: TablerIcon::Folder,
        shortcut: "MOD O",
    },
    SpotlightResult {
        id: "color-theme",
        label: "Switch color theme",
        description: "Appearance",
        icon: TablerIcon::Sun,
        shortcut: "MOD T",
    },
    SpotlightResult {
        id: "gpu-frame",
        label: "Inspect GPU frame",
        description: "Developer tools",
        icon: TablerIcon::Rust,
        shortcut: "MOD I",
    },
];

/// Search-oriented single-window showcase demonstrating filtering and global focus.
pub struct SpotlightShowcase {
    query: String,
    selected_result: usize,
    last_action: Option<String>,
    capabilities: Option<WindowCapabilities>,
    light_icons: WidgetAssets,
    dark_icons: WidgetAssets,
    selected_icons: WidgetAssets,
}

impl Default for SpotlightShowcase {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected_result: 0,
            last_action: None,
            capabilities: None,
            light_icons: WidgetAssets::tabler_subset(
                Color::from_srgb8(82, 82, 91),
                SPOTLIGHT_ICONS,
            ),
            dark_icons: WidgetAssets::tabler_subset(
                Color::from_srgb8(212, 212, 216),
                SPOTLIGHT_ICONS,
            ),
            selected_icons: WidgetAssets::tabler_subset(Color::WHITE, SPOTLIGHT_ICONS),
        }
    }
}

impl AppModel for SpotlightShowcase {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        (window == &WindowKey::main()).then(|| self.window(environment))
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Ui { window, event } => match (event.target_key(), &event.kind) {
                (Some(SEARCH_KEY), UiEventKind::TextChanged(value)) => {
                    self.query.clone_from(value);
                    self.selected_result = 0;
                    self.last_action = None;
                    rebuild(window)
                }
                (Some(key), UiEventKind::Click(_)) if key.starts_with(RESULT_KEY_PREFIX) => {
                    if self.activate_result(&key[RESULT_KEY_PREFIX.len()..]) {
                        rebuild(window)
                    } else {
                        AppUpdate::none()
                    }
                }
                (Some("minimize"), UiEventKind::Click(_)) => {
                    AppUpdate::none().command(AppCommand::MinimizeWindow(window.clone()))
                }
                (Some("hide"), UiEventKind::Click(_)) => {
                    AppUpdate::none().command(AppCommand::HideWindow(window.clone()))
                }
                _ => AppUpdate::none(),
            },
            AppEvent::Window {
                window,
                event: PlatformEvent::Opened { capabilities, .. },
            } => {
                self.capabilities = Some(*capabilities);
                let update = rebuild(window);
                if capabilities.window_level {
                    update.command(AppCommand::SetWindowLevel {
                        window: window.clone(),
                        level: argui_platform::WindowLevel::AlwaysOnTop,
                    })
                } else {
                    update
                }
            }
            AppEvent::Window {
                window,
                event: PlatformEvent::Keyboard(input),
            } if input.state == KeyState::Pressed => {
                if self.handle_launcher_key(input) {
                    rebuild(window)
                } else {
                    AppUpdate::none()
                }
            }
            AppEvent::Window { .. }
            | AppEvent::WindowReady { .. }
            | AppEvent::WindowFailed { .. }
            | AppEvent::Tray(_) => AppUpdate::none(),
            AppEvent::GlobalShortcut(event)
                if event.id.as_str() == SPOTLIGHT_SHORTCUT_ID
                    && event.state == GlobalShortcutState::Pressed =>
            {
                AppUpdate::none().command(AppCommand::FocusWindow(WindowKey::main()))
            }
            AppEvent::GlobalShortcut(_) => AppUpdate::none(),
        }
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.light_icons
            .assets()
            .iter()
            .chain(self.dark_icons.assets())
            .chain(self.selected_icons.assets())
            .cloned()
            .collect()
    }
}

impl SpotlightShowcase {
    /// Returns the result records matching the current controlled query.
    fn matching_results(&self) -> Vec<SpotlightResult> {
        let query = self.query.trim().to_lowercase();
        SPOTLIGHT_RESULTS
            .into_iter()
            .filter(|result| {
                query.is_empty()
                    || result.label.to_lowercase().contains(&query)
                    || result.description.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// Applies launcher navigation or activation for a window key press.
    fn handle_launcher_key(&mut self, input: &KeyInput) -> bool {
        let results = self.matching_results();
        if results.is_empty() {
            return false;
        }
        match input.key {
            Key::ArrowDown => {
                self.selected_result = (self.selected_result + 1) % results.len();
                true
            }
            Key::ArrowUp => {
                self.selected_result = (self.selected_result + results.len() - 1) % results.len();
                true
            }
            Key::Enter if !input.repeat => {
                let result = results[self.selected_result.min(results.len() - 1)];
                self.last_action = Some(format!("Activated · {}", result.label));
                true
            }
            _ => false,
        }
    }

    /// Selects and activates the visible result identified by `id`.
    fn activate_result(&mut self, id: &str) -> bool {
        let results = self.matching_results();
        let Some((index, result)) = results
            .into_iter()
            .enumerate()
            .find(|(_, result)| result.id == id)
        else {
            return false;
        };
        self.selected_result = index;
        self.last_action = Some(format!("Activated · {}", result.label));
        true
    }

    fn window(&self, environment: WindowEnvironment) -> Element {
        let theme = shadcn(&environment);
        let theme = theme.resolve(environment.color_scheme);
        let icons = match environment.color_scheme {
            ColorScheme::Light => &self.light_icons,
            ColorScheme::Dark => &self.dark_icons,
        };
        let glass = DesktopBackdrop::new(theme.card.with_alpha(0.76), theme.card.with_alpha(0.97))
            .inactive_tint(theme.card.with_alpha(0.88))
            .inactive_fallback(theme.card.with_alpha(0.985));
        let panel = Element::column([
            self.title_bar(theme, icons),
            self.results(theme, icons),
            self.footer(theme, icons),
        ])
        .keyed("spotlight-panel")
        .width(percent(1.0))
        .height(percent(1.0))
        .border(Border::all(1.0, theme.border))
        .desktop_backdrop(glass)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .focus_scope(FocusScope {
            containment: FocusContainment::None,
            initial: Some(InitialFocus::Target(FocusTarget::from(SEARCH_KEY))),
            restore: false,
        });
        let panel = AnimatedContainer::from_element(panel)
            .radius(CornerRadii::all(if self.query.is_empty() {
                PANEL_RADIUS
            } else {
                PANEL_RADIUS + 2.0
            }))
            .duration(Duration::from_millis(220))
            .curve(curves::EMPHASIZED)
            .build();
        Element::container([panel])
            .width(percent(1.0))
            .height(percent(1.0))
            .background(Color::TRANSPARENT)
    }

    fn title_bar(&self, theme: &argui_widgets::WidgetTheme, icons: &WidgetAssets) -> Element {
        let search_quad = QuadStyle::solid(Color::TRANSPARENT).radius(CornerRadii::all(13.0));
        let mut search_style = theme.input();
        search_style.layout.size.height = length(54.0);
        search_style.layout.padding = sides(50.0, 14.0);
        search_style.paint = PaintStyle::new(search_quad.clone());
        search_style.hovered = argui_ui::StylePatch::from_quad(search_quad.clone());
        search_style.focused = argui_ui::StylePatch::from_quad(search_quad);
        search_style.text.font_size = 17.0;
        search_style.text.line_height = 22.0;
        search_style.text.weight = 500;
        search_style.placeholder.font_size = 17.0;
        search_style.placeholder.line_height = 22.0;
        let search = Input::new(
            SEARCH_KEY,
            &self.query,
            "Search apps and commands…",
            search_style,
        )
        .kind(InputKind::Search)
        .label("Search apps and commands")
        .leading(icons.icon(TablerIcon::Search, 21.0), 48.0)
        .build();
        Element::container([search])
            .height(length(74.0))
            .padding(sides(10.0, 9.0))
            .border(Border {
                widths: BorderWidths {
                    bottom: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
    }

    fn results(&self, theme: &argui_widgets::WidgetTheme, icons: &WidgetAssets) -> Element {
        let rows = self
            .matching_results()
            .into_iter()
            .enumerate()
            .map(|(index, result)| {
                result_row(
                    (
                        result.id,
                        result.label,
                        result.description,
                        result.shortcut,
                        result.icon,
                    ),
                    index,
                    index == self.selected_result,
                    theme,
                    icons,
                    &self.selected_icons,
                )
            })
            .collect::<Vec<_>>();
        let count = rows.len();
        let query_empty = self.query.trim().is_empty();
        let heading = Element::row([
            Element::text(if query_empty {
                "SUGGESTIONS".to_owned()
            } else {
                format!("{count} RESULT{}", if count == 1 { "" } else { "S" })
            })
            .text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 10.0,
                line_height: 14.0,
                weight: 750,
                ..TextStyle::default()
            }),
            Element::text(self.backend_label()).text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 10.0,
                line_height: 14.0,
                weight: 500,
                ..TextStyle::default()
            }),
        ])
        .padding(sides(10.0, 2.0))
        .justify_content(JustifyContent::SPACE_BETWEEN);
        AnimatedContainer::from_element(
            Element::column([heading, Element::column(rows).gap(4.0).grow(1.0)])
                .keyed("spotlight-results")
                .padding(sides(12.0, 8.0))
                .gap(6.0)
                .grow(1.0),
        )
        .background(if query_empty {
            Color::TRANSPARENT
        } else {
            with_alpha(theme.primary, 0.035)
        })
        .radius(CornerRadii::all(if query_empty { 12.0 } else { 16.0 }))
        .duration(Duration::from_millis(180))
        .curve(curves::EASE_OUT)
        .build()
    }

    fn footer(&self, theme: &argui_widgets::WidgetTheme, icons: &WidgetAssets) -> Element {
        let status = self.last_action.as_deref().unwrap_or_else(|| {
            if self
                .capabilities
                .is_some_and(|capabilities| capabilities.backend == WindowBackend::Web)
            {
                "Web preview"
            } else {
                "ARGUI · Ctrl+Space reopens after hiding"
            }
        });
        let brand = Element::row([
            icons.icon(TablerIcon::Search, 13.0),
            Element::text(status)
                .text_style(TextStyle {
                    color: theme.muted_foreground,
                    font_size: 10.0,
                    line_height: 14.0,
                    weight: 750,
                    ..TextStyle::default()
                })
                .semantics(
                    Semantics::new(Role::Status)
                        .label(status)
                        .live(LiveRegion::Polite),
                ),
        ])
        .gap(7.0)
        .align_items(AlignItems::CENTER);
        let mut actions = vec![
            shortcut_hint("UP/DN", "Navigate", theme),
            shortcut_hint("ENTER", "Open", theme),
        ];
        if self
            .capabilities
            .is_some_and(|capabilities| capabilities.minimize)
        {
            actions.push(compact_button(
                "minimize",
                "Minimize",
                TablerIcon::Minus,
                theme,
                icons,
            ));
            actions.push(compact_button(
                "hide",
                "Hide Spotlight — Ctrl+Space to reopen",
                TablerIcon::ChevronDown,
                theme,
                icons,
            ));
        }
        Element::row([
            brand,
            Element::row(actions)
                .gap(7.0)
                .align_items(AlignItems::CENTER),
        ])
        .height(length(50.0))
        .padding(sides(18.0, 8.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .border(Border {
            widths: BorderWidths {
                top: 1.0,
                ..BorderWidths::default()
            },
            color: theme.border,
        })
    }

    fn backend_label(&self) -> String {
        let Some(capabilities) = self.capabilities else {
            return "Detecting the native window backend".into();
        };
        let backend = match capabilities.backend {
            WindowBackend::Windows => "Windows",
            WindowBackend::MacOs => "macOS",
            WindowBackend::Android => "Android",
            WindowBackend::Ios => "iOS",
            WindowBackend::X11 => "Linux · X11",
            WindowBackend::Wayland => "Linux · Wayland",
            WindowBackend::Web => "Web",
            WindowBackend::Other => "Other backend",
        };
        if capabilities.window_level {
            format!("{backend} · always on top")
        } else {
            backend.into()
        }
    }
}

fn rebuild(window: &WindowKey) -> AppUpdate {
    AppUpdate::none().window(window.clone(), ViewUpdate::Rebuild)
}
