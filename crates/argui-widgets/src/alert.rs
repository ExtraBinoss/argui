use argui_paint::{Border, CornerRadii};
use argui_text::TextStyle;
use argui_ui::{AlignItems, Element, LiveRegion, Role, Semantics, Sides, length, percent};

use crate::WidgetTheme;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

/// An inline message. Live announcements can be disabled for persistent content.
#[derive(Clone, Debug)]
pub struct Alert {
    key: String,
    title: String,
    description: Option<String>,
    icon: Option<Element>,
    variant: AlertVariant,
    live: LiveRegion,
}

impl Alert {
    /// Creates an assertively announced alert with the given title.
    ///
    /// `key` identifies the element and `title` provides its primary message.
    #[must_use]
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: None,
            icon: None,
            variant: AlertVariant::default(),
            live: LiveRegion::Assertive,
        }
    }

    #[must_use]
    /// Adds supporting text to the alert.
    /// `description` is the supporting text announced with the alert.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    /// Adds a decorative icon before the alert text.
    pub fn icon(mut self, icon: Element) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    /// Selects the alert's visual variant.
    pub const fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    /// Sets the live-region announcement behavior.
    pub const fn live(mut self, live: LiveRegion) -> Self {
        self.live = live;
        self
    }

    #[must_use]
    /// Builds the alert using `theme` for its colors and surface.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let (foreground, detail) = match self.variant {
            AlertVariant::Default => (theme.foreground, theme.muted_foreground),
            AlertVariant::Destructive => {
                let color = if theme.destructive.contrast_ratio(theme.card) >= 4.5 {
                    theme.destructive
                } else {
                    theme.destructive.mix(
                        theme.foreground,
                        0.5,
                        argui_core::ColorInterpolation::Oklab,
                    )
                };
                (color, color)
            }
        };
        let title_key = format!("{}::title", self.key);
        let description_key = format!("{}::description", self.key);
        let mut root = Element::row([])
            .keyed(self.key)
            .semantics(Semantics::new(Role::Alert).live(self.live))
            .labelled_by([title_key.clone()])
            .width(percent(1.0))
            .min_width(length(0.0))
            .align_items(AlignItems::START)
            .padding(Sides::length(16.0))
            .gap(12.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(8.0));
        let mut content = vec![
            Element::text(self.title)
                .keyed(title_key)
                .text_style(TextStyle {
                    font_size: 14.0,
                    line_height: 20.0,
                    weight: 600,
                    color: foreground,
                    ..TextStyle::default()
                }),
        ];
        if let Some(description) = self.description {
            root = root.described_by([description_key.clone()]);
            content.push(
                Element::text(description)
                    .keyed(description_key)
                    .text_style(TextStyle {
                        font_size: 14.0,
                        line_height: 21.0,
                        color: detail,
                        ..TextStyle::default()
                    }),
            );
        }
        root.children.extend(self.icon.map(|icon| {
            icon.vector_color(foreground)
                .shrink(0.0)
                .semantic_hidden(true)
        }));
        root.children.push(
            Element::column(content)
                .gap(4.0)
                .grow(1.0)
                .min_width(length(0.0)),
        );
        root
    }
}
