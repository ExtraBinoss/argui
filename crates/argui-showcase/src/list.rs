use argui_paint::{Border, Color, CornerRadii, QuadStyle};
use argui_text::{TextColor, TextWrap};
use argui_ui::{
    Edges, Element, Interaction, ScrollConfig, ScrollPolarity, ScrollbarStyle, VirtualList,
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
                    .scrollbar(self.scrollbar_style(self.accent())),
            )
    }

    pub(super) fn scrollbar_style(&self, accent: Color) -> ScrollbarStyle {
        ScrollbarStyle::new(
            QuadStyle::solid(Color::rgba(0.12, 0.16, 0.22, 0.72)).radius(CornerRadii::all(5.0)),
            QuadStyle::solid(accent).radius(CornerRadii::all(5.0)),
        )
        .width(10.0)
        .inset(5.0)
        .min_thumb(30.0)
    }

    pub(super) fn virtual_list(&self, accent: Color) -> Element {
        self.virtual_list_config()
            .build("million-list", self.virtual_offset, |index| {
                let background = if index % 2 == 0 {
                    Color::rgb(0.075, 0.10, 0.15)
                } else {
                    Color::rgb(0.06, 0.08, 0.12)
                };
                Element::text(format!("Row #{index:07} / 1,000,000"))
                    .keyed(format!("row-{index}"))
                    .text_style(text_style(
                        15.0,
                        TextColor::rgb(0.78, 0.84, 0.92),
                        500,
                        TextWrap::None,
                    ))
                    .padding(Edges::symmetric(12.0, 8.0))
                    .background(background)
                    .interaction(
                        Interaction::default().hovered(
                            QuadStyle::solid(Color::rgb(0.12, 0.18, 0.25))
                                .border(Border::all(1.0, accent)),
                        ),
                    )
            })
            .background(Color::rgb(0.035, 0.045, 0.065))
            .border(Border::all(1.0, Color::rgb(0.18, 0.24, 0.32)))
            .radius(CornerRadii::all(12.0))
    }
}
