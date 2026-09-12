use super::*;
use crate::app::text;
use argui::{
    ui::{length, percent},
    widgets::*,
};

impl CatalogueDemo {
    fn message(&self, index: usize, theme: &WidgetTheme) -> Element {
        let end = index.is_multiple_of(2);
        let mut bubble = Bubble::new(
            format!("bubble-{index}"),
            text(
                if end {
                    "Could you review the new layout?"
                } else {
                    "Yes, the spacing is clearer and the controls are easier to reach."
                },
                14.0,
                theme.foreground,
                400,
            ),
        );
        bubble.end = end;
        let mut message = Message::new(
            format!("message-{index}"),
            if end { "You" } else { "Ada" },
            bubble.build(theme),
        );
        message.header = Some(text(
            if end { "You · 10:42" } else { "Ada · 10:43" },
            12.0,
            theme.muted_foreground,
            500,
        ));
        message.end = end;
        if index == 1 {
            message.footer = Some(
                Button::new(
                    "message-like",
                    if self.open { "Liked" } else { "Like" },
                    theme.ghost_button(),
                )
                .build(),
            );
        }
        message.build()
    }

    fn message_scroller(&self, theme: &WidgetTheme) -> MessageScroller {
        let mut scroller = MessageScroller::new(
            "chat",
            "Conversation",
            Element::column((0..self.messages).map(|index| self.message(index, theme)))
                .keyed("chat-messages")
                .gap(18.0),
        );
        scroller.state = self.message_scroll.clone();
        scroller
    }

    pub(super) fn conversation_view(&self, theme: &WidgetTheme) -> Element {
        match self.page {
            Page::Attachment => Element::column(
                [
                    ("design.fig", AttachmentState::Done, None),
                    ("references.zip", AttachmentState::Uploading, Some(0.64)),
                    ("notes.pdf", AttachmentState::Error, None),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (name, state, progress))| {
                    let mut attachment = Attachment::new(format!("attachment-{index}"), name);
                    attachment.state = state;
                    attachment.progress = progress;
                    attachment.item.media = Some(text("FILE", 11.0, theme.muted_foreground, 600));
                    attachment.item.actions = Some(
                        Button::new(
                            format!("attachment-action-{index}"),
                            if state == AttachmentState::Error {
                                "Retry"
                            } else {
                                "Details"
                            },
                            theme.outline_button(),
                        )
                        .build(),
                    );
                    attachment.build(theme)
                }),
            )
            .gap(12.0),
            Page::Bubble => Element::column(
                [
                    BubbleVariant::Secondary,
                    BubbleVariant::Tinted,
                    BubbleVariant::Outline,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, variant)| {
                    let mut bubble = Bubble::new(
                        format!("sample-{index}"),
                        text(
                            [
                                "A compact message surface.",
                                "A softer accent for a reply.",
                                "A quiet outlined conversation.",
                            ][index],
                            14.0,
                            theme.foreground,
                            400,
                        ),
                    );
                    bubble.variant = variant;
                    bubble.end = index == 1;
                    bubble.build(theme)
                }),
            )
            .gap(18.0)
            .width(percent(1.0)),
            Page::Message => {
                Element::column([self.message(0, theme), self.message(1, theme)]).gap(18.0)
            }
            Page::Marker => Element::column(
                [
                    MarkerVariant::Inline,
                    MarkerVariant::Border,
                    MarkerVariant::Separator,
                ]
                .into_iter()
                .enumerate()
                .map(|(index, variant)| {
                    let mut marker = Marker::new(
                        format!("note-{index}"),
                        [
                            "All changes saved",
                            "Switched to the design branch",
                            "Today",
                        ][index],
                    );
                    marker.variant = variant;
                    marker.build(theme)
                }),
            )
            .gap(18.0),
            Page::Item => {
                let mut item = Item::new("project-item", "Website redesign");
                item.description = Some("12 screens · Updated today".into());
                item.media = Some(text("W", 24.0, theme.primary, 700));
                item.actions =
                    Some(Button::new("item-open", "Open", theme.outline_button()).build());
                item.build(theme)
            }
            Page::ScrollArea => ScrollArea::new(
                "documents",
                "Documents",
                280.0,
                Element::column((1..=24).map(|index| {
                    text(
                        format!("Document {index:02} · Project notes"),
                        14.0,
                        theme.foreground,
                        400,
                    )
                    .height(length(32.0))
                }))
                .gap(8.0),
            )
            .build(theme),
            Page::MessageScroller => Element::column([
                self.message_scroller(theme).build(theme),
                Button::new("chat-add", "Receive a message", theme.button()).build(),
            ])
            .gap(12.0),
            _ => unreachable!("conversation page"),
        }
    }

    pub(super) fn conversation_event(
        &mut self,
        event: &UiEvent,
        theme: &WidgetTheme,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.page == Page::MessageScroller {
            if let Some(MessageScrollerAction::Latest) = self.message_scroller(theme).action(event)
            {
                cx.scroll(self.message_scroll.latest("chat"));
                return true;
            }
            if ButtonBehavior::new("chat-add", "Receive a message")
                .action(event)
                .is_some()
            {
                self.messages += 1;
                self.append_count = Some(1);
                return true;
            }
            let previous = self.message_scroll.clone();
            self.message_scroll
                .observe(event, "chat", self.scroll_maximum);
            if self.message_scroll != previous {
                return true;
            }
        }
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return false;
        }
        match event.target_key() {
            Some("message-like") => {
                self.open = !self.open;
                self.status = if self.open {
                    "Message liked"
                } else {
                    "Reaction removed"
                }
                .into();
            }
            Some("item-open") => self.status = "Opened Website redesign".into(),
            Some(key) if key.starts_with("attachment-action-") => {
                self.status = "File details are ready.".into()
            }
            _ => return false,
        }
        true
    }
}
