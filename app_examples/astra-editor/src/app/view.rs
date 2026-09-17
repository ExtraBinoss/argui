use argui::{
    paint::{Border, BorderWidths},
    runtime::Context,
    ui::{
        AlignItems, Display, Element, FocusScope, InitialFocus, Interaction, JustifyContent, Role,
        Semantics, Sides, length, percent,
    },
    widgets::{Button, SplitAxis, SplitPane, TablerIcon, WidgetTheme},
};

use super::{AstraEditor, SearchMode};
use crate::ui::label;

impl AstraEditor {
    /// Composes the complete responsive application surface.
    pub(super) fn view(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let content = if self.compact {
            if self.explorer_revealed {
                self.project_sidebar(theme, cx)
                    .width(percent(1.0))
                    .grow(1.0)
            } else {
                self.editor_workspace(theme, cx)
            }
        } else {
            self.desktop_workspace(theme, cx)
        };
        let workspace = if self.compact {
            Element::column([
                content.grow(1.0).min_height(length(0.0)),
                self.activity_bar(theme, cx),
            ])
        } else {
            Element::row([
                self.activity_bar(theme, cx),
                content.grow(1.0).min_width(length(0.0)),
            ])
        };
        let application = Element::column([workspace.grow(1.0), self.status_bar(theme)])
            .width(percent(1.0))
            .height(percent(1.0))
            .min_width(length(0.0))
            .min_height(length(0.0));
        let mut layers = vec![application];
        if let Some(search) = self.search_overlay(theme, cx) {
            layers.push(search);
        }
        Element::container(layers)
            .keyed(match self.theme_mode {
                argui::theme::ThemeMode::Dark => "astra-editor-dark",
                argui::theme::ThemeMode::Light | argui::theme::ThemeMode::System => {
                    "astra-editor-light"
                }
            })
            .focus_scope(FocusScope {
                initial: Some(InitialFocus::Target("code-editor".into())),
                ..FocusScope::restoring().restore(false)
            })
            .interaction(Interaction::default().focus_policy(argui::ui::FocusPolicy::TabStop))
            .semantics(
                Semantics::new(Role::Window)
                    .label("Astra Editor")
                    .description("Responsive Rust project editor built with Argui"),
            )
            .width(percent(1.0))
            .height(percent(1.0))
            .min_width(length(0.0))
            .min_height(length(0.0))
            .background(theme.background)
            .inspectable(true)
    }

    /// Builds the resizable explorer and editor arrangement used on larger screens.
    fn desktop_workspace(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let pane = SplitPane::new(
            "workspace-explorer-split",
            SplitAxis::Horizontal,
            self.explorer_width,
            180.0,
            420.0,
        )
        .on_change(cx.value_callback(|editor, value| editor.explorer_width = value));
        let mut separator = pane.separator(theme);
        if let Some(semantics) = &mut separator.semantics {
            semantics.label = Some("Resize project explorer".into());
        }
        if !self.explorer_open {
            return Element::row([
                self.project_sidebar(theme, cx).display(Display::None),
                separator.display(Display::None),
                self.editor_workspace(theme, cx)
                    .grow(1.0)
                    .min_width(length(0.0))
                    .min_height(length(0.0)),
            ])
            .width(percent(1.0))
            .height(percent(1.0))
            .min_width(length(0.0))
            .min_height(length(0.0));
        }
        pane.build(
            self.project_sidebar(theme, cx),
            separator,
            self.editor_workspace(theme, cx),
            self.viewport.width,
            360.0,
        )
        .width(percent(1.0))
        .height(percent(1.0))
        .min_width(length(0.0))
        .min_height(length(0.0))
    }

    /// Builds the VS Code-style activity rail, horizontal on compact screens.
    fn activity_bar(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let explorer_active = if self.compact {
            self.explorer_revealed
        } else {
            self.explorer_open
        };
        let explorer = Button::icon(
            "toggle-explorer",
            "Explorer · Ctrl+B",
            self.assets
                .icon(TablerIcon::Sidebar, 19.0)
                .vector_color(if explorer_active {
                    theme.foreground
                } else {
                    theme.muted_foreground
                }),
            theme.ghost_button(),
        )
        .tooltip("Explorer · Ctrl+B")
        .on_click(cx.event_handler(|editor, _, cx| editor.toggle_explorer(cx)))
        .build()
        .width(length(40.0))
        .height(length(40.0))
        .padding(Sides::length(0.0))
        .background(if explorer_active {
            theme.muted
        } else {
            argui::core::Color::TRANSPARENT
        })
        .border(Border {
            widths: BorderWidths {
                left: if explorer_active && !self.compact {
                    2.0
                } else {
                    0.0
                },
                bottom: if explorer_active && self.compact {
                    2.0
                } else {
                    0.0
                },
                ..BorderWidths::default()
            },
            color: theme.primary,
        });
        let search = Button::icon(
            "global-search",
            "Search project · Ctrl+Shift+F",
            self.assets
                .icon(TablerIcon::Search, 19.0)
                .vector_color(theme.muted_foreground),
            theme.ghost_button(),
        )
        .tooltip("Search project · Ctrl+Shift+F")
        .on_click(cx.event_handler(|editor, _, cx| editor.show_search(SearchMode::Text, cx)))
        .build()
        .width(length(40.0))
        .height(length(40.0))
        .padding(Sides::length(0.0));
        let open = Button::icon(
            "open-project",
            if cfg!(target_arch = "wasm32") {
                "Import project files"
            } else {
                "Open project folder"
            },
            self.assets
                .icon(
                    if self.loading_project {
                        TablerIcon::Loader
                    } else {
                        TablerIcon::Folder
                    },
                    19.0,
                )
                .vector_color(theme.muted_foreground),
            theme.ghost_button(),
        )
        .tooltip(if cfg!(target_arch = "wasm32") {
            "Import project files"
        } else {
            "Open project folder"
        })
        .enabled(!self.loading_project)
        .on_click(cx.event_handler(|editor, _, cx| editor.choose_project(cx)))
        .build()
        .width(length(40.0))
        .height(length(40.0))
        .padding(Sides::length(0.0));
        let save = Button::icon(
            "save-document",
            "Save · Ctrl+S",
            self.assets
                .icon(TablerIcon::Check, 19.0)
                .vector_color(theme.muted_foreground),
            theme.ghost_button(),
        )
        .tooltip("Save · Ctrl+S")
        .enabled(self.active_document().is_dirty())
        .on_click(cx.event_handler(|editor, _, cx| editor.save_active(cx)))
        .build()
        .width(length(40.0))
        .height(length(40.0))
        .padding(Sides::length(0.0));
        let terminal = Button::icon(
            "activity-terminal",
            "Open terminal · Ctrl+J",
            self.assets
                .icon(TablerIcon::Terminal, 19.0)
                .vector_color(theme.muted_foreground),
            theme.ghost_button(),
        )
        .tooltip("Open terminal · Ctrl+J")
        .on_click(cx.event_handler(|editor, _, cx| editor.launch_terminal(cx)))
        .build()
        .width(length(40.0))
        .height(length(40.0))
        .padding(Sides::length(0.0));
        let theme_icon = match self.theme_mode {
            argui::theme::ThemeMode::Dark => TablerIcon::Sun,
            argui::theme::ThemeMode::Light | argui::theme::ThemeMode::System => TablerIcon::Moon,
        };
        let appearance = Button::icon(
            "toggle-theme",
            "Toggle light and dark theme",
            self.assets
                .icon(theme_icon, 17.0)
                .vector_color(theme.foreground),
            theme.ghost_button(),
        )
        .tooltip("Toggle light and dark theme")
        .on_click(cx.event_handler(|editor, _, cx| editor.toggle_theme(cx)))
        .build()
        .width(length(40.0))
        .height(length(40.0))
        .padding(Sides::length(0.0));
        let primary = vec![explorer, search, open, save];
        let secondary = vec![terminal, appearance];
        if self.compact {
            Element::row(primary.into_iter().chain(secondary))
                .keyed("editor-activity-bar")
                .height(length(46.0))
                .padding(Sides::length(3.0))
                .gap(3.0)
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::CENTER)
                .background(theme.card)
                .border(Border {
                    widths: BorderWidths {
                        top: 1.0,
                        ..BorderWidths::default()
                    },
                    color: theme.border,
                })
        } else {
            Element::column([
                Element::column(primary).gap(3.0),
                Element::column(secondary).gap(3.0),
            ])
            .keyed("editor-activity-bar")
            .width(length(46.0))
            .height(percent(1.0))
            .padding(Sides::length(3.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .background(theme.card)
            .border(Border {
                widths: BorderWidths {
                    right: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
        }
    }

    /// Builds the compact bottom status bar shared by every layout.
    fn status_bar(&self, theme: &WidgetTheme) -> Element {
        let document = self.active_document();
        let left = self.notice.as_deref().unwrap_or("Ready");
        let mut right = vec![label(
            document.language_label(),
            10.5,
            theme.muted_foreground,
            650,
        )];
        if !self.compact {
            right.insert(
                0,
                label(
                    format!("{} lines", document.line_count()),
                    10.5,
                    theme.muted_foreground,
                    500,
                ),
            );
            right.push(label("UTF-8", 10.5, theme.muted_foreground, 500));
        }
        Element::row([
            label(left, 10.5, theme.muted_foreground, 500)
                .min_width(length(0.0))
                .shrink(1.0),
            Element::row(right).gap(13.0).shrink(0.0),
        ])
        .keyed("editor-status")
        .height(length(26.0))
        .padding(Sides {
            left: length(12.0),
            right: length(12.0),
            top: length(4.0),
            bottom: length(4.0),
        })
        .gap(12.0)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .background(theme.card)
        .border(Border {
            widths: BorderWidths {
                top: 1.0,
                ..BorderWidths::default()
            },
            color: theme.border,
        })
    }
}
