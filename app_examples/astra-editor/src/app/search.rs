use argui::{
    core::Color,
    paint::{Border, CornerRadii, LayerMask, LayerStyle, Shadow},
    runtime::Context,
    ui::{
        AlignItems, Element, EventType, FocusScope, InitialFocus, Interaction, JustifyContent,
        Role, Semantics, Sides, length, percent,
    },
    widgets::{Button, Input, InputKind, Kbd, TablerIcon, VList, WidgetTheme},
};

use super::{AstraEditor, SearchMode};
use crate::ui::{accent, label, scroll_shadow};

impl AstraEditor {
    /// Builds the animated global-search overlay while it is entering, open, or exiting.
    pub(super) fn search_overlay(
        &self,
        theme: &WidgetTheme,
        cx: &mut Context<Self>,
    ) -> Option<Element> {
        if !self.search.open && self.search.progress == 0.0 {
            return None;
        }
        let progress = self.search_progress();
        let panel_width = if self.compact { 0.96 } else { 0.72 };
        let result_viewport =
            (self.viewport.height - if self.compact { 210.0 } else { 260.0 }).clamp(180.0, 500.0);
        let panel = Element::column([
            self.search_header(theme, cx),
            self.search_mode_switcher(theme, cx),
            self.search_results(result_viewport, theme, cx),
            self.search_footer(theme),
        ])
        .keyed("workspace-search-panel")
        .width(percent(panel_width))
        .max_width(length(820.0))
        .min_width(length(0.0))
        .max_height(length(if self.compact {
            self.viewport.height - 24.0
        } else {
            640.0
        }))
        .background(theme.popover)
        .border(Border::all(1.0, theme.popover_border))
        .radius(CornerRadii::all(if self.compact { 13.0 } else { 16.0 }))
        .layer(
            LayerStyle::new(Default::default())
                .shadow(Shadow::glow(24.0, Color::BLACK.with_alpha(0.18)))
                .mask(LayerMask::Rounded(CornerRadii::all(if self.compact {
                    13.0
                } else {
                    16.0
                }))),
        )
        .semantics(Semantics::new(Role::Dialog).label(self.search.mode.title()));
        let panel = panel
            .opacity(progress)
            .transform(argui::core::Transform2D::IDENTITY.translate(0.0, 8.0 * (1.0 - progress)));
        let backdrop = Element::container([])
            .keyed("workspace-search-backdrop")
            .absolute(Sides::length(0.0))
            .background(Color::BLACK.with_alpha(0.44 * progress))
            .interaction(Interaction::blocker())
            .on(cx
                .event_handler(|editor, event, cx| {
                    editor.hide_search(cx);
                    event.stop_propagation();
                })
                .listener(EventType::Click));
        let panel_layer = Element::row([panel])
            .width(percent(1.0))
            .height(percent(1.0))
            .padding(Sides {
                left: length(0.0),
                right: length(0.0),
                top: length(if self.compact { 12.0 } else { 28.0 }),
                bottom: length(12.0),
            })
            .align_items(AlignItems::START)
            .justify_content(JustifyContent::CENTER)
            .absolute(Sides::length(0.0))
            .z_index(1);
        Some(
            Element::container([backdrop, panel_layer])
                .keyed("workspace-search-overlay")
                .focus_scope(FocusScope::modal(InitialFocus::Target(
                    "workspace-search".into(),
                )))
                .interaction(Interaction::blocker())
                .width(percent(1.0))
                .height(percent(1.0))
                .absolute(Sides::length(0.0))
                .z_index(100),
        )
    }

    /// Builds the search title, query input, and close action.
    fn search_header(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let input = Input::new(
            "workspace-search",
            &self.search.query,
            self.search.mode.placeholder(),
            theme.input(),
        )
        .kind(InputKind::Search)
        .label(self.search.mode.title())
        .leading(
            self.assets
                .icon(TablerIcon::Search, 17.0)
                .vector_color(theme.muted_foreground),
            40.0,
        )
        .on_input(cx.input_callback(|editor, value| {
            editor.search.query = value;
            editor.refresh_search();
        }))
        .on_submit(cx.submit_event_handler(|editor, _, _, cx| {
            editor.activate_first_search_result(cx);
        }))
        .build()
        .grow(1.0)
        .min_width(length(0.0));
        let close = Button::icon(
            "close-search",
            "Close search",
            self.assets
                .icon(TablerIcon::Close, 17.0)
                .vector_color(theme.foreground),
            theme.ghost_button(),
        )
        .on_click(cx.event_handler(|editor, _, cx| editor.hide_search(cx)))
        .build()
        .width(length(36.0))
        .height(length(36.0))
        .padding(Sides::length(0.0));
        Element::row([input, close])
            .padding(Sides::length(if self.compact { 10.0 } else { 14.0 }))
            .gap(8.0)
            .align_items(AlignItems::CENTER)
    }

    /// Builds the text/file search mode selector.
    fn search_mode_switcher(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let modes = [
            (SearchMode::Text, "Text", "Search file contents"),
            (SearchMode::Files, "Files", "Open a file by path"),
        ];
        Element::row(modes.into_iter().map(|(mode, title, description)| {
            let style = if self.search.mode == mode {
                theme.button()
            } else {
                theme.ghost_button()
            };
            Button::new(format!("search-mode::{mode:?}"), title, style)
                .tooltip(description)
                .on_click(cx.event_handler(move |editor, _, cx| {
                    editor.search.mode = mode;
                    editor.refresh_search();
                    cx.request_focus("workspace-search");
                    cx.notify();
                }))
                .build()
                .height(length(29.0))
        }))
        .padding(Sides {
            left: length(if self.compact { 10.0 } else { 14.0 }),
            right: length(if self.compact { 10.0 } else { 14.0 }),
            top: length(0.0),
            bottom: length(10.0),
        })
        .gap(5.0)
    }

    /// Builds the virtualized content or file result list.
    fn search_results(
        &self,
        viewport: f32,
        theme: &WidgetTheme,
        cx: &mut Context<Self>,
    ) -> Element {
        let count = match self.search.mode {
            SearchMode::Text => self.search.text_matches.len(),
            SearchMode::Files => self.search.file_matches.len(),
        };
        if count == 0 {
            return self.empty_search(viewport, theme);
        }
        let row_height = match self.search.mode {
            SearchMode::Text => 60.0,
            SearchMode::Files => 46.0,
        };
        VList::new(
            "workspace-search-results",
            row_height,
            viewport,
            self.search.offset,
        )
        .effect(scroll_shadow(theme))
        .build(count, theme, |index| match self.search.mode {
            SearchMode::Text => self.text_result(index, theme, cx),
            SearchMode::Files => self.file_result(index, theme, cx),
        })
        .height(length(viewport))
        .border(Border {
            widths: argui::paint::BorderWidths {
                top: 1.0,
                bottom: 1.0,
                ..argui::paint::BorderWidths::default()
            },
            color: theme.border,
        })
    }

    /// Builds a text-search result that opens its source document.
    fn text_result(&self, index: usize, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let result = &self.search.text_matches[index];
        let document = &self.project.documents[result.document];
        let document_id = result.document;
        let content = Element::column([
            Element::row([
                label(&document.path, 11.0, theme.foreground, 620),
                label(
                    format!("line {}", result.line),
                    10.0,
                    theme.muted_foreground,
                    500,
                ),
            ])
            .gap(8.0)
            .align_items(AlignItems::CENTER),
            highlighted_preview(&result.preview, &self.search.query, theme),
        ])
        .gap(3.0)
        .min_width(length(0.0));
        Button::new(
            format!("search-result::{index}"),
            format!("Open {} at line {}", document.path, result.line),
            theme.ghost_button(),
        )
        .content(content)
        .on_click(cx.event_handler(move |editor, _, cx| {
            editor.open_document(document_id, cx);
            editor.hide_search(cx);
        }))
        .build()
        .width(percent(1.0))
        .height(length(58.0))
        .justify_content(JustifyContent::START)
        .radius(CornerRadii::all(0.0))
    }

    /// Builds one quick-open file result.
    fn file_result(&self, index: usize, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let document_id = self.search.file_matches[index];
        let document = &self.project.documents[document_id];
        let icon = if document.extension() == "rs" {
            TablerIcon::Rust
        } else {
            TablerIcon::File
        };
        let content = Element::row([
            self.assets.icon(icon, 15.0).vector_color(accent(1.0)),
            highlighted_preview(&document.path, &self.search.query, theme),
        ])
        .gap(10.0)
        .align_items(AlignItems::CENTER);
        Button::new(
            format!("file-result::{index}"),
            format!("Open {}", document.path),
            theme.ghost_button(),
        )
        .content(content)
        .on_click(cx.event_handler(move |editor, _, cx| {
            editor.open_document(document_id, cx);
            editor.hide_search(cx);
        }))
        .build()
        .width(percent(1.0))
        .height(length(44.0))
        .justify_content(JustifyContent::START)
        .radius(CornerRadii::all(0.0))
    }

    /// Builds the no-results state with shortcut discovery.
    fn empty_search(&self, viewport: f32, theme: &WidgetTheme) -> Element {
        let message = if self.search.query.trim().is_empty() {
            match self.search.mode {
                SearchMode::Text => format!(
                    "Search {} indexed files. Results update as you type.",
                    self.project.documents.len()
                ),
                SearchMode::Files => "Start typing or choose any indexed file below.".into(),
            }
        } else {
            format!("No match for “{}”", self.search.query)
        };
        Element::column([
            self.assets
                .icon(TablerIcon::Search, 25.0)
                .vector_color(theme.muted_foreground),
            label(message, 12.0, theme.muted_foreground, 500),
            Kbd::new("search-empty-shortcut", ["Ctrl", "⇧", "F"])
                .label("Control Shift F")
                .size(20.0)
                .build(theme),
        ])
        .height(length(viewport))
        .padding(Sides::length(24.0))
        .gap(10.0)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)
        .border(Border {
            widths: argui::paint::BorderWidths {
                top: 1.0,
                bottom: 1.0,
                ..argui::paint::BorderWidths::default()
            },
            color: theme.border,
        })
    }

    /// Builds search performance metadata and the dismiss keycap.
    fn search_footer(&self, theme: &WidgetTheme) -> Element {
        let count = match self.search.mode {
            SearchMode::Text => self.search.text_matches.len(),
            SearchMode::Files => self.search.file_matches.len(),
        };
        Element::row([
            label(
                format!(
                    "{count} results · {} µs · in-memory index",
                    self.search.elapsed_micros
                ),
                10.5,
                theme.muted_foreground,
                500,
            ),
            Kbd::new("dismiss-search", ["Esc"])
                .label("Escape")
                .size(19.0)
                .build(theme),
        ])
        .height(length(40.0))
        .padding(Sides {
            left: length(14.0),
            right: length(14.0),
            top: length(8.0),
            bottom: length(8.0),
        })
        .gap(12.0)
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
    }

    /// Opens the first current search result when the query is submitted.
    fn activate_first_search_result(&mut self, cx: &mut Context<Self>) {
        let document = match self.search.mode {
            SearchMode::Text => self
                .search
                .text_matches
                .first()
                .map(|result| result.document),
            SearchMode::Files => self.search.file_matches.first().copied(),
        };
        if let Some(document) = document {
            self.open_document(document, cx);
            self.hide_search(cx);
        }
    }
}

/// Builds a preview with the first ASCII query match tinted in the editor accent.
fn highlighted_preview(preview: &str, query: &str, theme: &WidgetTheme) -> Element {
    let query = query.trim();
    let found = query
        .is_ascii()
        .then(|| {
            preview
                .to_ascii_lowercase()
                .find(&query.to_ascii_lowercase())
        })
        .flatten();
    let Some(start) = found else {
        return label(preview, 11.0, theme.muted_foreground, 450);
    };
    let end = start + query.len();
    Element::row([
        label(&preview[..start], 11.0, theme.muted_foreground, 450),
        label(&preview[start..end], 11.0, accent(1.0), 700),
        label(&preview[end..], 11.0, theme.muted_foreground, 450),
    ])
    .min_width(length(0.0))
}
