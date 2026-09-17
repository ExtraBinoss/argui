use super::{PRIMARIES, WidgetGallery, mode_label};
use crate::pages;
use argui::{
    core::Color,
    paint::{Border, BorderWidths, CornerRadii, ImageFit},
    runtime::{Context, WindowEnvironment},
    theme::ThemeMode,
    ui::{
        AlignItems, Axes, CursorIcon, Element, GestureSet, Interaction, JustifyContent,
        KeyboardActivation, Overflow, Role, ScrollConfig, SemanticAction, Semantics, Sides, length,
        percent,
    },
    widgets::{Button, TablerIcon, WidgetAssets, WidgetTheme},
};

impl WidgetGallery {
    pub(super) fn view(
        &self,
        environment: WindowEnvironment,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
        resize: pages::ResizeListeners,
    ) -> Element {
        let content = self.content(theme, assets, cx, resize);
        let body = if self.compact {
            Element::column([self.mobile_navigation(theme, assets, cx), content])
                .grow(1.0)
                .min_height(length(0.0))
        } else {
            Element::row([self.sidebar(theme, assets, cx), content])
                .grow(1.0)
                .min_height(length(0.0))
        };
        Element::column([
            self.topbar(theme, assets, environment, cx),
            body,
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

    fn content(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
        resize: pages::ResizeListeners,
    ) -> Element {
        let mut content = Element::container([pages::render(self, theme, assets, cx, resize)])
            .keyed("gallery-content-scroll")
            .background(theme.background)
            .grow(1.0)
            .min_width(length(0.0))
            .min_height(length(0.0))
            .padding(Sides::length(if self.compact { 14.0 } else { 30.0 }))
            .overflow(Axes {
                x: if self.compact {
                    Overflow::Auto
                } else {
                    Overflow::Hidden
                },
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()));
        content = if self.compact {
            content.width(percent(1.0))
        } else {
            content.width(length(0.0))
        };
        content
    }

    fn topbar(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        environment: WindowEnvironment,
        cx: &mut Context<Self>,
    ) -> Element {
        let brand = Element::row([
            Element::image(self.logo)
                .image_fit(ImageFit::Contain)
                .width(length(34.0))
                .height(length(34.0))
                .semantics(Semantics::new(Role::Image).label("Argui Astra logo")),
            Element::column([
                super::text("ARGUI", 16.0, theme.foreground, 750),
                super::text("Widget Gallery", 12.0, theme.muted_foreground, 500),
            ])
            .gap(1.0),
        ])
        .align_items(AlignItems::CENTER)
        .gap(10.0);
        let controls =
            (!self.compact).then(|| self.desktop_controls(theme, assets, environment, cx));
        Element::row(std::iter::once(brand).chain(controls))
            .keyed("gallery-topbar")
            .height(length(64.0))
            .padding(Sides {
                left: length(if self.compact { 14.0 } else { 22.0 }),
                right: length(if self.compact { 14.0 } else { 126.0 }),
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

    fn desktop_controls(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        environment: WindowEnvironment,
        cx: &mut Context<Self>,
    ) -> Element {
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
        let mut controls = vec![swatches];
        if cfg!(feature = "desktop-backdrop") {
            controls.push(self.backdrop_controls(theme, environment));
        }
        controls.push(self.theme_button(theme, assets, cx));
        Element::row(controls).gap(10.0)
    }

    pub(super) fn theme_button(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
    ) -> Element {
        let icon = match self.theme_mode {
            ThemeMode::Light => TablerIcon::Sun,
            ThemeMode::Dark => TablerIcon::Moon,
            ThemeMode::System => TablerIcon::System,
        };
        Button::new(
            "theme-mode",
            mode_label(self.theme_mode),
            theme.ghost_button(),
        )
        .leading(assets.icon(icon, 17.0))
        .on_click(cx.event_handler(|gallery, _, cx| {
            gallery.cycle_theme();
            cx.set_theme(gallery.theme_request());
        }))
        .build()
    }
}
