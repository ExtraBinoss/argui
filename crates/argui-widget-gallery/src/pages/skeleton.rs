use argui::{
    runtime::{Context, Entity, Render},
    ui::{AlignItems, Element, EventType, Role, SemanticState, Semantics, length, percent},
    widgets::{Avatar, Button, Card, Skeleton, shadcn},
};

use crate::app::text;

pub(crate) struct SkeletonDemo {
    loaded: bool,
    cover: Entity<Skeleton>,
    avatar: Entity<Skeleton>,
    title: Entity<Skeleton>,
    subtitle: Entity<Skeleton>,
}

impl Default for SkeletonDemo {
    fn default() -> Self {
        Self {
            loaded: false,
            cover: Entity::new(Skeleton::new("skeleton-cover").size(percent(1.0), length(160.0))),
            avatar: Entity::new(
                Skeleton::new("skeleton-avatar")
                    .size(length(44.0), length(44.0))
                    .radius(999.0),
            ),
            title: Entity::new(Skeleton::new("skeleton-title").size(percent(0.8), length(18.0))),
            subtitle: Entity::new(
                Skeleton::new("skeleton-subtitle").size(percent(0.6), length(14.0)),
            ),
        }
    }
}

impl Render for SkeletonDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let content = if self.loaded {
            Element::column([
                Element::column([
                    text("A place for your next idea.", 26.0, theme.foreground, 600),
                    text(
                        "A few thoughtful details make all the difference.",
                        14.0,
                        theme.muted_foreground,
                        400,
                    ),
                ])
                .gap(10.0)
                .padding(argui::ui::Sides::length(24.0))
                .height(length(160.0))
                .background(theme.muted)
                .radius(argui::paint::CornerRadii::all(6.0)),
                Element::row([
                    Avatar::new("loaded-avatar", "Ada Lovelace", "AL")
                        .size(44.0)
                        .build(theme),
                    Element::column([
                        text("Designing something new", 15.0, theme.foreground, 600),
                        text(
                            "Ada Lovelace · 5 min read",
                            13.0,
                            theme.muted_foreground,
                            400,
                        ),
                    ])
                    .gap(6.0)
                    .min_width(length(0.0)),
                ])
                .gap(14.0)
                .align_items(AlignItems::CENTER),
            ])
            .gap(20.0)
        } else {
            Element::column([
                cx.entity(&self.cover),
                Element::row([
                    cx.entity(&self.avatar),
                    Element::column([cx.entity(&self.title), cx.entity(&self.subtitle)])
                        .gap(10.0)
                        .grow(1.0)
                        .min_width(length(0.0)),
                ])
                .gap(14.0)
                .align_items(AlignItems::CENTER),
            ])
            .gap(20.0)
        };
        let card = Card::new("skeleton-card", content).build(theme).semantics(
            Semantics::new(Role::Group)
                .label(if self.loaded {
                    "Article preview"
                } else {
                    "Loading article"
                })
                .state(SemanticState {
                    busy: !self.loaded,
                    ..SemanticState::default()
                }),
        );
        super::preview(
            "Leave room for what comes next",
            "Preview the loading state, then reveal the finished content.",
            Element::column([
                card,
                Button::new(
                    "skeleton-toggle",
                    if self.loaded {
                        "Show loading"
                    } else {
                        "Show content"
                    },
                    theme.outline_button(),
                )
                .build()
                .on(cx.listener(EventType::Click, |demo, _, cx| {
                    demo.loaded = !demo.loaded;
                    cx.notify();
                })),
            ])
            .gap(20.0)
            .max_width(length(480.0)),
            theme,
        )
    }
}
