use argui_ui::{AlignItems, Element, FlexDirection, Role, Semantics, length, percent};

/// Conversation row with arbitrary content, avatar, header and footer slots.
#[derive(Clone, Debug)]
pub struct Message {
    pub key: String,
    pub author: String,
    pub content: Element,
    pub avatar: Option<Element>,
    pub header: Option<Element>,
    pub footer: Option<Element>,
    pub end: bool,
}

impl Message {
    /// Creates a message identified by `key`, with an `author` label and `content` element.
    #[must_use]
    pub fn new(key: impl Into<String>, author: impl Into<String>, content: Element) -> Self {
        Self {
            key: key.into(),
            author: author.into(),
            content,
            avatar: None,
            header: None,
            footer: None,
            end: false,
        }
    }

    #[must_use]
    /// Builds the message presentation.
    pub fn build(self) -> Element {
        let content = Element::column(
            self.header
                .into_iter()
                .chain([self.content])
                .chain(self.footer),
        )
        .gap(6.0)
        .min_width(length(0.0))
        .grow(1.0)
        .align_items(if self.end {
            AlignItems::END
        } else {
            AlignItems::START
        });
        Element::row(
            self.avatar
                .into_iter()
                .map(|avatar| avatar.semantic_hidden(true).shrink(0.0))
                .chain([content]),
        )
        .keyed(self.key)
        .width(percent(1.0))
        .gap(10.0)
        .align_items(AlignItems::START)
        .flex_direction(if self.end {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .semantics(Semantics::new(Role::Group).label(self.author))
    }
}
