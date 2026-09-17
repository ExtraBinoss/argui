use argui::{
    paint::{Border, BorderWidths, CornerRadii, PaintStyle, QuadStyle},
    runtime::Context,
    text::{FontFamily, TextStyle, TextWrap},
    ui::{
        AlignItems, Axes, Element, JustifyContent, Overflow, ScrollAxes, ScrollConfig,
        ScrollPropagation, Sides, StylePatch, TextSelectionHighlight, length, percent,
    },
    widgets::{Button, TablerIcon, TextArea, WidgetTheme},
};

use super::AstraEditor;
use crate::ui::{accent, label, scroll_shadow};

impl AstraEditor {
    /// Builds the editor workspace; terminal actions launch the platform's real terminal.
    pub(super) fn editor_workspace(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        self.editor_surface(theme, cx)
    }

    /// Builds tabs, the file toolbar, and the controlled multiline code editor.
    fn editor_surface(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        Element::column([
            self.tab_strip(theme, cx),
            self.file_toolbar(theme),
            self.code_editor(theme, cx),
        ])
        .keyed("editor-surface")
        .width(percent(1.0))
        .height(percent(1.0))
        .min_width(length(0.0))
        .min_height(length(0.0))
        .background(theme.background)
    }

    /// Builds horizontally scrollable open-file tabs with retained dirty state.
    fn tab_strip(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let tabs = self.open_documents.iter().map(|document| {
            let document_id = *document;
            let file = &self.project.documents[document_id];
            let active = document_id == self.active_document;
            let select = cx.event_handler(move |editor, _, cx| {
                editor.open_document(document_id, cx);
            });
            let close = cx.event_handler(move |editor, event, cx| {
                editor.close_document(document_id, cx);
                event.stop_propagation();
            });
            let dirty = file.is_dirty().then(|| {
                Element::container([])
                    .width(length(6.0))
                    .height(length(6.0))
                    .background(accent(1.0))
                    .radius(CornerRadii::all(99.0))
            });
            let content = Element::row(dirty.into_iter().chain(std::iter::once(label(
                file.name(),
                11.5,
                if active {
                    theme.foreground
                } else {
                    theme.muted_foreground
                },
                if active { 620 } else { 500 },
            ))))
            .gap(7.0)
            .align_items(AlignItems::CENTER);
            let mut style = if active {
                theme.outline_button()
            } else {
                theme.ghost_button()
            };
            style.paint = PaintStyle::new(
                QuadStyle::solid(if active { theme.background } else { theme.card })
                    .border(Border {
                        widths: BorderWidths {
                            bottom: if active { 2.0 } else { 0.0 },
                            ..BorderWidths::default()
                        },
                        color: accent(1.0),
                    })
                    .radius(CornerRadii::all(7.0)),
            );
            let tab = Button::new(
                format!("tab::{document_id}"),
                format!("Open {}", file.name()),
                style,
            )
            .content(content)
            .on_click(select)
            .build()
            .height(length(30.0));
            let close = Button::icon(
                format!("close-tab::{document_id}"),
                format!("Close {}", file.name()),
                self.assets
                    .icon(TablerIcon::Close, 13.0)
                    .vector_color(theme.muted_foreground),
                theme.ghost_button(),
            )
            .on_click(close)
            .build()
            .width(length(26.0))
            .height(length(26.0))
            .padding(Sides::length(0.0));
            Element::row([tab, close])
                .gap(1.0)
                .align_items(AlignItems::CENTER)
                .padding(Sides {
                    left: length(2.0),
                    right: length(2.0),
                    top: length(3.0),
                    bottom: length(3.0),
                })
                .shrink(0.0)
        });
        let strip = Element::row(tabs)
            .padding(Sides {
                left: length(7.0),
                right: length(7.0),
                top: length(1.0),
                bottom: length(1.0),
            })
            .gap(2.0)
            .background(theme.card)
            .shrink(0.0);
        Element::layout_boundary(strip)
            .keyed("open-tabs")
            .width(percent(1.0))
            .height(length(38.0))
            .min_width(length(0.0))
            .shrink(0.0)
            .background(theme.card)
            .border(Border {
                widths: BorderWidths {
                    bottom: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
            .overflow(Axes {
                x: Overflow::Auto,
                y: Overflow::Hidden,
            })
            .scroll_config(
                ScrollConfig::default()
                    .axes(ScrollAxes::Horizontal)
                    .propagation(ScrollPropagation::Contain)
                    .scrollbar(theme.scrollbar.clone())
                    .effect(scroll_shadow(theme)),
            )
    }

    /// Builds the active file breadcrumb and editor-dock controls.
    fn file_toolbar(&self, theme: &WidgetTheme) -> Element {
        let document = self.active_document();
        let state = if document.is_dirty() {
            "Edited"
        } else {
            "Saved"
        };
        let breadcrumb = Element::row([
            label(&document.path, 11.0, theme.muted_foreground, 500)
                .min_width(length(0.0))
                .shrink(1.0),
            label(
                state,
                10.0,
                if document.is_dirty() {
                    accent(1.0)
                } else {
                    theme.muted_foreground
                },
                650,
            ),
        ])
        .gap(9.0)
        .align_items(AlignItems::CENTER)
        .min_width(length(0.0));
        Element::row([breadcrumb])
            .height(length(36.0))
            .padding(Sides {
                left: length(if self.compact { 10.0 } else { 16.0 }),
                right: length(if self.compact { 6.0 } else { 10.0 }),
                top: length(4.0),
                bottom: length(4.0),
            })
            .gap(8.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::START)
            .border(Border {
                widths: BorderWidths {
                    bottom: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
    }

    /// Builds the monospaced text area with non-wrapping code and rounded selection fragments.
    fn code_editor(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let document = self.active_document();
        let mut style = theme.input();
        let quad = QuadStyle::solid(theme.background);
        style.paint = PaintStyle::new(quad.clone());
        style.hovered = StylePatch::from_quad(quad.clone());
        style.focused = StylePatch::from_quad(quad);
        style.layout.padding = argui::ui::sides(if self.compact { 13.0 } else { 20.0 }, 16.0);
        style.text = TextStyle {
            color: theme.foreground,
            font_size: if self.compact { 13.0 } else { 13.5 },
            line_height: if self.compact { 20.0 } else { 21.0 },
            family: FontFamily::Monospace,
            weight: 430,
            wrap: TextWrap::None,
            ..TextStyle::default()
        };
        style.placeholder = style.text.clone();
        style.placeholder.color = theme.muted_foreground;
        style.selection = accent(0.28);
        let mut editor = TextArea::new("code-editor", &document.content, "Start writing…", style)
            .wrap(TextWrap::None)
            .scroll_config(
                ScrollConfig::default()
                    .propagation(ScrollPropagation::Contain)
                    .scrollbar(theme.scrollbar.clone())
                    .effect(scroll_shadow(theme)),
            );
        if let Some(content) =
            document.highlighted_content(matches!(self.theme_mode, argui::theme::ThemeMode::Dark))
        {
            editor = editor.rich_text(content);
        }
        editor
            .on_edit(cx.edit_event_handler(|editor, edit, _, cx| {
                editor.edit_active(edit, cx);
            }))
            .build()
            .selection_highlight(TextSelectionHighlight::solid(accent(0.28)).radius(5.0))
            .width(percent(1.0))
            .min_width(length(0.0))
            .min_height(length(0.0))
            .flex_basis(length(0.0))
            .grow(1.0)
            .shrink(1.0)
    }
}
