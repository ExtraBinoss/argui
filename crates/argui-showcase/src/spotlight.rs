use argui_animation::{Duration, Frame};
use argui_core::Color;
use argui_paint::{Border, BorderWidths, CornerRadii, PaintStyle, QuadStyle};
use argui_platform::{PlatformEvent, WindowBackend, WindowCapabilities, WindowKey};
use argui_runtime::{AppCommand, AppEvent, AppModel, AppUpdate, ViewUpdate, WindowEnvironment};
use argui_text::TextStyle;
use argui_ui::{
    AlignItems, Axes, Element, FocusContainment, FocusScope, FocusTarget, InitialFocus,
    Interaction, JustifyContent, Overflow, Sides, UiEventKind, WindowDragBehavior, length, percent,
    sides,
};
use argui_widgets::{Button, ButtonStyle, Input, shadcn};

const SEARCH_KEY: &str = "spotlight-search";
const PASSTHROUGH_TIME: Duration = Duration::from_secs(2);
const PANEL_WIDTH: f32 = 688.0;
const PANEL_HEIGHT: f32 = 428.0;
const PANEL_RADIUS: f32 = 16.0;
pub const SPOTLIGHT_WINDOW_WIDTH: f64 = PANEL_WIDTH as f64;
pub const SPOTLIGHT_WINDOW_HEIGHT: f64 = PANEL_HEIGHT as f64;

#[derive(Default)]
pub struct SpotlightShowcase {
    query: String,
    capabilities: Option<WindowCapabilities>,
    passthrough_remaining: Option<Duration>,
}

impl AppModel for SpotlightShowcase {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        (window == &WindowKey::main()).then(|| self.window(environment))
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Ui { window, event } => match (event.key.as_deref(), &event.kind) {
                (Some(SEARCH_KEY), UiEventKind::TextChanged(value)) => {
                    self.query.clone_from(value);
                    rebuild(window)
                }
                (Some("minimize"), UiEventKind::Clicked) => {
                    AppUpdate::none().command(AppCommand::MinimizeWindow(window.clone()))
                }
                (Some("close"), UiEventKind::Clicked) => {
                    AppUpdate::none().command(AppCommand::Quit)
                }
                (Some("passthrough"), UiEventKind::Clicked)
                    if self.passthrough_remaining.is_none() =>
                {
                    self.passthrough_remaining = Some(PASSTHROUGH_TIME);
                    rebuild(window).command(AppCommand::SetWindowMousePassthrough {
                        window: window.clone(),
                        passthrough: true,
                    })
                }
                _ => AppUpdate::none(),
            },
            AppEvent::Window {
                window,
                event: PlatformEvent::Opened { capabilities, .. },
            } => {
                self.capabilities = Some(*capabilities);
                rebuild(window)
            }
            AppEvent::Window { .. } | AppEvent::Tray(_) => AppUpdate::none(),
        }
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        let Some(remaining) = self.passthrough_remaining else {
            return AppUpdate::none();
        };
        let remaining = remaining - frame.elapsed;
        if remaining == Duration::ZERO {
            self.passthrough_remaining = None;
            rebuild(window).command(AppCommand::SetWindowMousePassthrough {
                window: window.clone(),
                passthrough: false,
            })
        } else {
            self.passthrough_remaining = Some(remaining);
            AppUpdate::none()
        }
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        window == &WindowKey::main() && self.passthrough_remaining.is_some()
    }
}

impl SpotlightShowcase {
    fn window(&self, environment: WindowEnvironment) -> Element {
        let theme = shadcn(environment.primary);
        let theme = theme.resolve(environment.color_scheme);
        let panel = Element::column([
            self.title_bar(theme),
            Element::column([
                Input::new(
                    SEARCH_KEY,
                    &self.query,
                    "Search commands, files and settings",
                    theme.input.clone(),
                )
                .build(),
                self.results(theme),
                self.footer(theme),
            ])
            .padding(Sides::length(18.0))
            .gap(14.0)
            .grow(1.0),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(PANEL_RADIUS))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .focus_scope(FocusScope {
            containment: FocusContainment::None,
            initial: Some(InitialFocus::Target(FocusTarget::from(SEARCH_KEY))),
            restore: false,
        });
        Element::container([panel])
            .width(percent(1.0))
            .height(percent(1.0))
    }

    fn title_bar(&self, theme: &argui_widgets::WidgetTheme) -> Element {
        let title = Element::column([
            Element::text("Argui Spotlight").text_style(TextStyle {
                color: theme.foreground,
                font_size: 14.0,
                line_height: 18.0,
                weight: 650,
                ..TextStyle::default()
            }),
            Element::text(self.backend_label()).text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 11.0,
                line_height: 14.0,
                ..TextStyle::default()
            }),
        ])
        .gap(1.0);
        let controls = Element::row([
            compact_button("minimize", "Minimize", theme),
            compact_button("close", "Close", theme),
        ])
        .gap(8.0);
        Element::row([title, controls])
            .height(length(58.0))
            .padding(sides(18.0, 12.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .border(Border {
                widths: BorderWidths {
                    bottom: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
            .interaction(
                Interaction::default().window_drag(WindowDragBehavior::MoveAndToggleMaximize),
            )
    }

    fn results(&self, theme: &argui_widgets::WidgetTheme) -> Element {
        let query = self.query.trim().to_lowercase();
        let candidates = [
            ("Open command palette", "Navigation"),
            ("Toggle DevTools", "Developer"),
            ("Switch color theme", "Appearance"),
            ("Inspect GPU frame", "Performance"),
        ];
        let rows = candidates
            .into_iter()
            .filter(|(label, category)| {
                query.is_empty()
                    || label.to_lowercase().contains(&query)
                    || category.to_lowercase().contains(&query)
            })
            .map(|(label, category)| result_row(label, category, theme));
        Element::column(rows)
            .background(with_alpha(theme.muted, 0.62))
            .radius(CornerRadii::all(10.0))
            .padding(Sides::length(6.0))
            .gap(3.0)
            .grow(1.0)
    }

    fn footer(&self, theme: &argui_widgets::WidgetTheme) -> Element {
        let active = self.passthrough_remaining.is_some();
        let status = if active {
            "Input passes through this overlay for two seconds"
        } else {
            "Drag the header · double-click it to maximize"
        };
        Element::row([
            Element::text(status).text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 12.0,
                line_height: 16.0,
                ..TextStyle::default()
            }),
            Button::new(
                "passthrough",
                if active {
                    "Click-through active"
                } else {
                    "Click-through 2 s"
                },
                compact_style(theme),
            )
            .build(),
        ])
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
    }

    fn backend_label(&self) -> String {
        let Some(capabilities) = self.capabilities else {
            return "Detecting the native window backend".into();
        };
        let backend = match capabilities.backend {
            WindowBackend::Windows => "Windows",
            WindowBackend::MacOs => "macOS",
            WindowBackend::X11 => "Linux · X11",
            WindowBackend::Wayland => "Linux · Wayland",
            WindowBackend::Web => "Web",
            WindowBackend::Other => "Other backend",
        };
        if capabilities.window_level {
            format!("{backend} · always on top")
        } else {
            format!("{backend} · always on top unavailable")
        }
    }
}

fn result_row(label: &str, category: &str, theme: &argui_widgets::WidgetTheme) -> Element {
    Element::row([
        Element::text(label).text_style(TextStyle {
            color: theme.foreground,
            font_size: 14.0,
            line_height: 20.0,
            weight: 550,
            ..TextStyle::default()
        }),
        Element::text(category).text_style(TextStyle {
            color: theme.muted_foreground,
            font_size: 12.0,
            line_height: 18.0,
            ..TextStyle::default()
        }),
    ])
    .padding(sides(10.0, 9.0))
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::SPACE_BETWEEN)
}

fn compact_button(key: &str, label: &str, theme: &argui_widgets::WidgetTheme) -> Element {
    Button::new(key, label, compact_style(theme)).build()
}

fn compact_style(theme: &argui_widgets::WidgetTheme) -> ButtonStyle {
    let mut style = ButtonStyle::new(
        PaintStyle::new(
            QuadStyle::solid(theme.card)
                .border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(7.0)),
        ),
        TextStyle {
            color: theme.foreground,
            font_size: 12.0,
            line_height: 16.0,
            weight: 600,
            ..TextStyle::default()
        },
    );
    style.layout.padding = sides(10.0, 7.0);
    style.hovered = argui_ui::StylePatch::from_quad(
        QuadStyle::solid(theme.muted).radius(CornerRadii::all(7.0)),
    );
    style.pressed = argui_ui::StylePatch::from_quad(
        QuadStyle::solid(theme.muted)
            .radius(CornerRadii::all(7.0))
            .opacity(0.76),
    );
    style
}

fn rebuild(window: &WindowKey) -> AppUpdate {
    AppUpdate::none().window(window.clone(), ViewUpdate::Rebuild)
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let [red, green, blue, _] = color.as_array();
    Color::rgba(red, green, blue, alpha)
}
