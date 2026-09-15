use crate::WidgetTheme;
use argui_paint::{Border, CornerRadii};
use argui_text::{FontFamily, TextStyle};
use argui_ui::{Element, Role, Semantics, Sides};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Semantic style choice for [`Typography`].
pub enum TypographyVariant {
    Heading(u32),
    #[default]
    Paragraph,
    Lead,
    Large,
    Small,
    Muted,
    Code,
    Quote,
}

/// Semantic themed text; selections and rich inline composition remain engine responsibilities.
#[derive(Clone, Debug)]
pub struct Typography {
    pub text: String,
    pub variant: TypographyVariant,
}

impl Typography {
    /// Creates semantic text with the requested visual variant.
    ///
    /// `text` is the displayed content; `variant` selects its typography and semantics.
    #[must_use]
    pub fn new(text: impl Into<String>, variant: TypographyVariant) -> Self {
        Self {
            text: text.into(),
            variant,
        }
    }

    /// Builds the text element using the supplied theme colors.
    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let (size, weight) = match self.variant {
            TypographyVariant::Heading(level) => (
                match level {
                    0 | 1 => 36.0,
                    2 => 30.0,
                    3 => 24.0,
                    _ => 20.0,
                },
                700,
            ),
            TypographyVariant::Lead => (20.0, 400),
            TypographyVariant::Large => (18.0, 600),
            TypographyVariant::Small => (14.0, 500),
            TypographyVariant::Muted | TypographyVariant::Code => (14.0, 400),
            TypographyVariant::Paragraph | TypographyVariant::Quote => (16.0, 400),
        };
        let mut semantics = Semantics::new(Role::Text).label(&self.text);
        if let TypographyVariant::Heading(level) = self.variant {
            semantics.role = Role::Heading;
            semantics.level = Some(level.clamp(1, 6));
        }
        let element = Element::text(self.text)
            .text_style(TextStyle {
                font_size: size,
                line_height: size * 1.5,
                weight,
                color: if matches!(
                    self.variant,
                    TypographyVariant::Lead | TypographyVariant::Muted
                ) {
                    theme.muted_foreground
                } else {
                    theme.foreground
                },
                family: if self.variant == TypographyVariant::Code {
                    FontFamily::Monospace
                } else {
                    FontFamily::SansSerif
                },
                ..TextStyle::default()
            })
            .semantics(semantics);
        match self.variant {
            TypographyVariant::Code => element
                .padding(Sides::length(4.0))
                .background(theme.muted)
                .radius(CornerRadii::all(4.0)),
            TypographyVariant::Quote => element
                .padding(Sides::length(12.0))
                .border(Border::all(1.0, theme.border)),
            _ => element,
        }
    }
}
