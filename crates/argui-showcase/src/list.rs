use argui_paint::{Border, Color, CornerRadii, QuadStyle};
use argui_text::TextWrap;
use argui_theme::WidgetTheme;
use argui_ui::{
    Edges, Element, Interaction, ScrollConfig, ScrollPolarity, ScrollbarPartStyle, ScrollbarStyle,
    StateStyle, VirtualList, VisualState,
};

use super::{StateShowcase, text_style};

impl StateShowcase {
    pub(super) fn virtual_list_config(&self) -> VirtualList {
        let polarity = if self.inverted_scroll {
            ScrollPolarity::Inverted
        } else {
            ScrollPolarity::Normal
        };
        VirtualList::new(1_000_000, 36.0, 260.0)
            .overscan(3)
            .scroll_config(
                ScrollConfig::default()
                    .polarity(polarity)
                    .line_size(36.0)
                    .scrollbar(self.scrollbar_style_from_colors(
                        Color::rgba(0.5, 0.5, 0.5, 0.16),
                        self.accent(),
                    )),
            )
    }

    fn scrollbar_style_from_colors(&self, track: Color, thumb: Color) -> ScrollbarStyle {
        ScrollbarStyle::new(
            ScrollbarPartStyle::new(QuadStyle::solid(track).radius(CornerRadii::all(5.0))),
            ScrollbarPartStyle::new(QuadStyle::solid(thumb).radius(CornerRadii::all(5.0))),
        )
        .width(10.0)
        .insets(Edges::all(5.0))
        .min_thumb(30.0)
    }

    pub(super) fn scrollbar_style(&self, widgets: &WidgetTheme) -> ScrollbarStyle {
        self.scrollbar_style_from_colors(widgets.muted, widgets.primary)
    }

    pub(super) fn virtual_list(&self, widgets: &WidgetTheme) -> Element {
        self.virtual_list_config()
            .scroll_config(
                ScrollConfig::default()
                    .polarity(if self.inverted_scroll {
                        ScrollPolarity::Inverted
                    } else {
                        ScrollPolarity::Normal
                    })
                    .line_size(36.0)
                    .scrollbar(self.scrollbar_style(widgets)),
            )
            .build("million-list", self.virtual_offset, |index| {
                let background = if index % 2 == 0 {
                    widgets.muted
                } else {
                    widgets.card
                };
                Element::text(format!("Row #{index:07} / 1,000,000"))
                    .keyed(format!("row-{index}"))
                    .text_style(text_style(15.0, widgets.foreground, 500, TextWrap::None))
                    .padding(Edges::symmetric(12.0, 8.0))
                    .background(background)
                    .interaction(Interaction::default())
                    .state(
                        VisualState::Hovered,
                        StateStyle::from_quad(
                            QuadStyle::solid(widgets.muted)
                                .border(Border::all(1.0, widgets.primary)),
                        ),
                    )
            })
            .background(widgets.card)
            .border(Border::all(1.0, widgets.border))
            .radius(CornerRadii::all(10.0))
    }
}
