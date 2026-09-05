use argui::{
    core::ColorScheme,
    paint::{Border, BorderWidths, CornerRadii, ImageFit, ImageId},
    runtime::{Context, Render},
    ui::{
        AlignItems, Axes, Element, EventListener, EventType, JustifyContent, Overflow, Role,
        Semantics, Sides, length, percent, sides,
    },
    widgets::{Button, SplitAxis, SplitPane, TablerIcon, WidgetAssets, WidgetTheme, shadcn},
};

use crate::app::text;

const MIN_WIDTH: f32 = 72.0;
const MAX_WIDTH: f32 = 300.0;
const COMPACT_WIDTH: f32 = 118.0;

fn render(
    split: &SplitPane,
    logo: ImageId,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    resize_listener: EventListener,
    reset_listener: EventListener,
) -> Element {
    let width = split.size;
    let compact = width <= COMPACT_WIDTH;
    let sidebar = Element::column([
        brand(compact, logo, theme),
        Element::column([
            navigation_item(
                "shell-home",
                "Overview",
                TablerIcon::System,
                compact,
                theme,
                assets,
            ),
            navigation_item(
                "shell-search",
                "Search",
                TablerIcon::Search,
                compact,
                theme,
                assets,
            ),
            navigation_item(
                "shell-projects",
                "Projects",
                TablerIcon::Sidebar,
                compact,
                theme,
                assets,
            ),
        ])
        .gap(6.0),
    ])
    .keyed("shell-sidebar")
    .width(length(width))
    .min_width(length(MIN_WIDTH))
    .max_width(length(MAX_WIDTH))
    .height(percent(1.0))
    .shrink(0.0)
    .padding(if compact {
        sides(12.0, 14.0)
    } else {
        Sides::length(14.0)
    })
    .gap(20.0)
    .background(theme.card)
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    });

    Element::row([
        sidebar,
        split
            .separator(theme)
            .on(resize_listener)
            .on(reset_listener),
        content(theme),
    ])
    .keyed("resizable-app-shell")
    .width(percent(1.0))
    .height(length(310.0))
    .min_width(length(360.0))
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(12.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    })
}

fn brand(compact: bool, logo: ImageId, theme: &WidgetTheme) -> Element {
    let mark = Element::image(logo)
        .image_fit(ImageFit::Contain)
        .semantics(Semantics::new(Role::Image).label("Argui logo"))
        .width(length(34.0))
        .height(length(34.0))
        .shrink(0.0)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)
        .radius(CornerRadii::all(9.0));
    let mut children = vec![mark];
    if !compact {
        children.push(Element::column([
            text("Astra", 14.0, theme.foreground, 700),
            text("Workspace", 11.0, theme.muted_foreground, 500),
        ]));
    }
    Element::row(children)
        .width(percent(1.0))
        .align_items(AlignItems::CENTER)
        .justify_content(if compact {
            JustifyContent::CENTER
        } else {
            JustifyContent::START
        })
        .gap(10.0)
}

fn navigation_item(
    key: &str,
    label: &str,
    icon: TablerIcon,
    compact: bool,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
) -> Element {
    let mut children = vec![assets.icon(icon, 18.0)];
    if !compact {
        children.push(text(label, 13.0, theme.foreground, 550));
    }
    Button::new(key, label, theme.ghost_button())
        .content(Element::row(children).gap(9.0))
        .build()
        .width(percent(1.0))
        .height(length(38.0))
        .padding(if compact {
            Sides::length(0.0)
        } else {
            sides(10.0, 0.0)
        })
        .gap(9.0)
        .align_items(AlignItems::CENTER)
        .justify_content(if compact {
            JustifyContent::CENTER
        } else {
            JustifyContent::START
        })
        .background(if key == "shell-home" {
            theme.muted
        } else {
            theme.card
        })
        .radius(CornerRadii::all(7.0))
}

fn content(theme: &WidgetTheme) -> Element {
    Element::column([
        Element::row([
            Element::column([
                text("Project overview", 18.0, theme.foreground, 700),
                text(
                    "Responsive application layout",
                    12.0,
                    theme.muted_foreground,
                    450,
                ),
            ]),
            text("Live", 11.0, theme.primary, 700),
        ])
        .width(percent(1.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN),
        Element::row([
            metric("Frames", "60 fps", theme),
            metric("GPU", "2.4 ms", theme),
        ])
        .width(percent(1.0))
        .gap(12.0),
        Element::container([])
            .width(percent(1.0))
            .grow(1.0)
            .background(theme.muted)
            .border(Border {
                widths: BorderWidths::all(1.0),
                color: theme.border,
            })
            .radius(CornerRadii::all(9.0)),
    ])
    .min_width(length(0.0))
    .grow(1.0)
    .padding(Sides::length(20.0))
    .gap(16.0)
    .background(theme.background)
}

fn metric(label: &str, value: &str, theme: &WidgetTheme) -> Element {
    Element::column([
        text(label, 11.0, theme.muted_foreground, 600),
        text(value, 16.0, theme.foreground, 700),
    ])
    .grow(1.0)
    .padding(Sides::length(12.0))
    .gap(3.0)
    .background(theme.card)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(8.0))
}

pub(crate) struct AppShell {
    split: SplitPane,
    logo: ImageId,
    light: WidgetAssets,
    dark: WidgetAssets,
}

impl AppShell {
    pub(crate) fn new(logo: ImageId, light: WidgetAssets, dark: WidgetAssets) -> Self {
        Self {
            split: SplitPane::new(
                "shell-sidebar-resize",
                SplitAxis::Horizontal,
                224.0,
                MIN_WIDTH,
                MAX_WIDTH,
            ),
            logo,
            light,
            dark,
        }
    }
}

impl Render for AppShell {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let themes = shadcn(environment.primary);
        let theme = themes.resolve(environment.color_scheme);
        let assets = if environment.color_scheme == ColorScheme::Dark {
            &self.dark
        } else {
            &self.light
        };
        let listener = cx.listener(EventType::Gesture, |shell, event, cx| {
            if shell.split.update(event) {
                cx.notify();
            }
        });
        let reset = cx.listener(EventType::Click, |shell, event, cx| {
            if shell.split.update(event) {
                cx.notify();
            }
        });
        render(&self.split, self.logo, theme, assets, listener, reset).on(cx
            .listener(EventType::Key, |shell, event, cx| {
                if shell.split.update(event) {
                    cx.notify();
                }
            })
            .capture(true))
    }
}
