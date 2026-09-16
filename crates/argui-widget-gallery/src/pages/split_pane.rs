use argui::{
    core::Color,
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    ui::{AlignItems, Element, Sides, length, percent, sides},
    widgets::{SplitAxis, SplitPane, WidgetTheme, shadcn},
};

/// Stateful split-pane demonstrations covering common application layouts.
pub(crate) struct SplitPaneDemo {
    navigator: f32,
    preview: f32,
    inspector: f32,
    ide_navigator: f32,
    ide_console: f32,
}

impl Default for SplitPaneDemo {
    fn default() -> Self {
        Self {
            navigator: 220.0,
            preview: 118.0,
            inspector: 245.0,
            ide_navigator: 175.0,
            ide_console: 105.0,
        }
    }
}

impl Render for SplitPaneDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column([
            super::preview(
                "Explorer and editor",
                "Drag the vertical divider, use Left/Right, or press Home/End. Double-click resets the initial size.",
                horizontal_example(self.navigator, theme, cx),
                theme,
            ),
            super::preview(
                "Preview over console",
                "A vertical split uses the same controlled API and Up/Down keyboard behavior.",
                vertical_example(self.preview, theme, cx),
                theme,
            ),
            super::preview(
                "Trailing inspector",
                "The fixed pane can live after the divider, so drag direction and keyboard controls remain natural.",
                trailing_example(self.inspector, theme, cx),
                theme,
            ),
            super::preview(
                "Nested IDE workspace",
                "Horizontal and vertical split panes compose without special layout primitives.",
                nested_example(self.ide_navigator, self.ide_console, theme, cx),
                theme,
            ),
        ])
        .width(percent(1.0))
        .gap(28.0)
    }
}

/// Builds the leading file-explorer split and reports its controlled width.
fn horizontal_example(size: f32, theme: &WidgetTheme, cx: &mut Context<SplitPaneDemo>) -> Element {
    let pane = SplitPane::new("split-navigator", SplitAxis::Horizontal, size, 120.0, 420.0)
        .on_change(cx.value_callback(|demo, value| demo.navigator = value));
    let separator = labeled_separator(&pane, theme, "Resize file explorer");
    Element::column([
        size_badge(format!("Explorer: {size:.0}px"), theme),
        pane.build(
            panel(
                "FILES",
                "src\n  app.rs\n  workspace.rs\ntests\n  interaction.rs",
                theme.primary,
                theme,
            ),
            separator,
            panel(
                "EDITOR · workspace.rs",
                "pub fn workspace() -> Element {\n    split.build(sidebar, editor)\n}",
                Color::from_srgb8(124, 58, 237),
                theme,
            ),
            840.0,
            180.0,
        )
        .height(length(210.0)),
    ])
    .width(percent(1.0))
    .gap(10.0)
}

/// Builds a vertically resizable preview and console stack.
fn vertical_example(size: f32, theme: &WidgetTheme, cx: &mut Context<SplitPaneDemo>) -> Element {
    let pane = SplitPane::new("split-preview", SplitAxis::Vertical, size, 72.0, 210.0)
        .on_change(cx.value_callback(|demo, value| demo.preview = value));
    let separator = labeled_separator(&pane, theme, "Resize preview and console");
    Element::column([
        size_badge(format!("Preview: {size:.0}px"), theme),
        pane.build(
            panel(
                "LIVE PREVIEW",
                "The rendered application updates above its build output.",
                Color::from_srgb8(5, 150, 105),
                theme,
            ),
            separator,
            panel(
                "CONSOLE",
                "Finished dev build in 184ms\nListening on localhost:3100",
                Color::from_srgb8(234, 88, 12),
                theme,
            ),
            270.0,
            60.0,
        )
        .height(length(270.0)),
    ])
    .width(percent(1.0))
    .gap(10.0)
}

/// Builds a canvas whose fixed inspector pane follows the separator.
fn trailing_example(size: f32, theme: &WidgetTheme, cx: &mut Context<SplitPaneDemo>) -> Element {
    let pane = SplitPane::new("split-inspector", SplitAxis::Horizontal, size, 150.0, 380.0)
        .trailing(true)
        .on_change(cx.value_callback(|demo, value| demo.inspector = value));
    let separator = labeled_separator(&pane, theme, "Resize trailing inspector");
    Element::column([
        size_badge(format!("Inspector: {size:.0}px"), theme),
        pane.build(
            panel(
                "CANVAS",
                "Select a layer to edit its layout, paint and interaction properties.",
                Color::from_srgb8(8, 145, 178),
                theme,
            ),
            separator,
            panel(
                "INSPECTOR",
                "Layout  Flex\nWidth   Fill\nGap     12px\nRadius  14px",
                Color::from_srgb8(219, 39, 119),
                theme,
            ),
            840.0,
            180.0,
        )
        .height(length(210.0)),
    ])
    .width(percent(1.0))
    .gap(10.0)
}

/// Builds a nested horizontal workspace and trailing vertical terminal split.
fn nested_example(
    navigator_size: f32,
    console_size: f32,
    theme: &WidgetTheme,
    cx: &mut Context<SplitPaneDemo>,
) -> Element {
    let console = SplitPane::new(
        "split-ide-console",
        SplitAxis::Vertical,
        console_size,
        64.0,
        190.0,
    )
    .trailing(true)
    .on_change(cx.value_callback(|demo, value| demo.ide_console = value));
    let console_separator = labeled_separator(&console, theme, "Resize IDE console");
    let editor_stack = console.build(
        panel(
            "main.rs",
            "fn main() {\n    argui::run(App::default());\n}",
            Color::from_srgb8(37, 99, 235),
            theme,
        ),
        console_separator,
        panel(
            "TERMINAL",
            "$ cargo nextest run\nPASS 1358 tests",
            Color::from_srgb8(5, 150, 105),
            theme,
        ),
        310.0,
        100.0,
    );
    let navigator = SplitPane::new(
        "split-ide-navigator",
        SplitAxis::Horizontal,
        navigator_size,
        110.0,
        320.0,
    )
    .on_change(cx.value_callback(|demo, value| demo.ide_navigator = value));
    let navigator_separator = labeled_separator(&navigator, theme, "Resize IDE navigator");
    Element::column([
        Element::row([
            size_badge(format!("Navigator: {navigator_size:.0}px"), theme),
            size_badge(format!("Console: {console_size:.0}px"), theme),
        ])
        .gap(8.0),
        navigator
            .build(
                panel(
                    "PROJECT",
                    "crates\n  argui-ui\n  argui-widgets\nwebsite\n  app",
                    Color::from_srgb8(234, 88, 12),
                    theme,
                ),
                navigator_separator,
                editor_stack,
                840.0,
                220.0,
            )
            .height(length(310.0)),
    ])
    .width(percent(1.0))
    .gap(10.0)
}

/// Builds a separator and gives it a scenario-specific accessible label.
fn labeled_separator(pane: &SplitPane, theme: &WidgetTheme, label: &str) -> Element {
    let mut separator = pane.separator(theme);
    separator
        .semantics
        .as_mut()
        .expect("SplitPane separators always expose semantics")
        .label = Some(label.to_owned());
    separator
}

/// Builds a compact application surface used as one side of a split.
fn panel(title: &str, detail: &str, accent: Color, theme: &WidgetTheme) -> Element {
    Element::column([
        Element::row([
            Element::container([])
                .width(length(8.0))
                .height(length(8.0))
                .background(accent)
                .radius(CornerRadii::all(999.0)),
            crate::app::text(title, 11.0, theme.foreground, 760),
        ])
        .align_items(AlignItems::CENTER)
        .gap(8.0),
        crate::app::text(detail, 13.0, theme.muted_foreground, 450),
    ])
    .width(percent(1.0))
    .height(percent(1.0))
    .min_width(length(0.0))
    .min_height(length(0.0))
    .padding(Sides::length(14.0))
    .gap(12.0)
    .background(accent.with_alpha(0.07))
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(10.0))
}

/// Displays the latest controlled pane size above its example.
fn size_badge(label: String, theme: &WidgetTheme) -> Element {
    crate::app::text(label, 11.0, theme.muted_foreground, 650)
        .padding(sides(9.0, 5.0))
        .background(theme.muted)
        .radius(CornerRadii::all(999.0))
        .align_self(argui::ui::AlignSelf::START)
}
