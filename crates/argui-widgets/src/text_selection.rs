use argui_core::{Rect, Size};
use argui_paint::{Border, CornerRadii, Filter, LayerMask, PaintStyle, QuadStyle};
use argui_ui::{Element, Interaction, SelectionCapabilities, Sides, UserSelect, length};

use crate::{Button, TablerIcon, WidgetAssets, WidgetTheme};
mod host;
pub use host::SelectionHost;

const TOOLBAR_HEIGHT: f32 = 40.0;
const COMMAND_WIDTH: f32 = 112.0;
const VIEWPORT_MARGIN: f32 = 8.0;
const SELECTION_GAP: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct TextSelectionToolbar {
    key_prefix: String,
    selection: Rect,
    viewport: Size,
    capabilities: SelectionCapabilities,
    icons: Option<WidgetAssets>,
    backdrop: Option<Filter>,
}

impl TextSelectionToolbar {
    #[must_use]
    pub fn new(
        key_prefix: impl Into<String>,
        selection: Rect,
        viewport: Size,
        capabilities: SelectionCapabilities,
    ) -> Self {
        Self {
            key_prefix: key_prefix.into(),
            selection,
            viewport,
            capabilities,
            icons: None,
            backdrop: None,
        }
    }

    #[must_use]
    pub fn icons(mut self, icons: &WidgetAssets) -> Self {
        self.icons = Some(icons.clone());
        self
    }

    /// The caller registers custom effects in the renderer's effect registry.
    #[must_use]
    pub fn backdrop_filter(mut self, filter: Filter) -> Self {
        self.backdrop = Some(filter);
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let commands = [
            ("cut", "Cut", self.capabilities.cut),
            ("copy", "Copy", self.capabilities.copy),
            ("paste", "Paste", self.capabilities.paste),
            ("select-all", "Select all", self.capabilities.select_all),
        ]
        .into_iter()
        .filter(|(command, _, _)| {
            self.capabilities.editable || matches!(*command, "copy" | "select-all")
        })
        .collect::<Vec<_>>();
        let count = commands.len().max(1) as f32;
        let command_width =
            COMMAND_WIDTH.min((self.viewport.width - VIEWPORT_MARGIN * 2.0).max(0.0) / count);
        let toolbar_width = command_width * count;
        let left = (self.selection.origin.x + self.selection.size.width * 0.5
            - toolbar_width * 0.5)
            .clamp(
                VIEWPORT_MARGIN,
                (self.viewport.width - toolbar_width - VIEWPORT_MARGIN).max(VIEWPORT_MARGIN),
            );
        let above = self.selection.origin.y - TOOLBAR_HEIGHT - SELECTION_GAP;
        let top = if above >= VIEWPORT_MARGIN {
            above
        } else {
            (self.selection.origin.y + self.selection.size.height + SELECTION_GAP)
                .min((self.viewport.height - TOOLBAR_HEIGHT - VIEWPORT_MARGIN).max(VIEWPORT_MARGIN))
        };
        let buttons = commands.into_iter().map(|(suffix, label, enabled)| {
            let mut button = Button::new(
                format!("{}::{suffix}", self.key_prefix),
                label,
                theme.ghost_button(),
            )
            .enabled(enabled);
            if let Some(icons) = &self.icons {
                let icon = match suffix {
                    "cut" => TablerIcon::Cut,
                    "copy" => TablerIcon::Copy,
                    "paste" => TablerIcon::Paste,
                    _ => TablerIcon::SelectAll,
                };
                let icon = icons.icon(icon, 14.0).vector_color(theme.foreground);
                button = if command_width < 100.0 {
                    button.content(icon)
                } else {
                    button.leading(icon)
                };
            }
            button
                .build()
                .width(length(command_width))
                .height(length(TOOLBAR_HEIGHT))
                .padding(argui_ui::sides(8.0, 6.0))
        });
        Element::row(buttons)
            .keyed(self.key_prefix)
            .portal(argui_ui::WindowLayer::Popover)
            .portal_dismiss(argui_ui::DismissPolicy::OutsidePointer)
            .absolute(Sides {
                left: length(left),
                right: argui_ui::auto(),
                top: length(top),
                bottom: argui_ui::auto(),
            })
            .width(length(toolbar_width))
            .height(length(TOOLBAR_HEIGHT))
            .paint_style(PaintStyle::new(
                QuadStyle::solid(theme.popover.with_alpha(if self.backdrop.is_some() {
                    0.08
                } else {
                    0.88
                }))
                .border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(8.0)),
            ))
            .backdrop_filter(
                self.backdrop
                    .unwrap_or(Filter::Blur(theme.overlay_blur.max(0.0))),
            )
            .mask(LayerMask::Rounded(CornerRadii::all(8.0)))
            .user_select(UserSelect::None)
            .interaction(Interaction::blocker())
            .z_index(i32::MAX)
    }
}
