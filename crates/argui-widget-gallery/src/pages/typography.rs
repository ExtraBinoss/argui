use std::num::NonZeroUsize;

use argui::{
    paint::{Border, CornerRadii},
    text::{
        EllipsisPosition, FontStretch, FontStyle, LetterSpacing, TextContent, TextDecoration,
        TextOverflow, TextSpan, TextSpanStyle, TextStyle, TextWrap, UnderlineStyle,
    },
    ui::{Element, Sides, TextSelectionStyle, UserSelect, length, percent},
    widgets::WidgetTheme,
};

use super::preview;

pub(super) fn render(theme: &WidgetTheme) -> Element {
    Element::column([
        preview(
            "Typography components",
            "A consistent scale for headings, paragraphs and supporting text.",
            Element::column([
                argui::widgets::Typography::new("A place for your ideas", argui::widgets::TypographyVariant::Heading(2)).build(theme),
                argui::widgets::Typography::new("Create, refine and share your next project.", argui::widgets::TypographyVariant::Lead).build(theme),
                argui::widgets::Typography::new("Every detail counts.", argui::widgets::TypographyVariant::Paragraph).build(theme),
                argui::widgets::Typography::new("Last edited just now", argui::widgets::TypographyVariant::Muted).build(theme),
            ]).gap(12.0),
            theme,
        ),
        preview(
            "Rich text",
            "One shaped paragraph can mix typography, colors and font metrics without splitting layout nodes.",
            rich_text(theme),
            theme,
        ),
        preview(
            "Font axes and decoration",
            "Tracking, style, stretch, underline and strike are native text properties.",
            type_styles(theme),
            theme,
        ),
        preview(
            "Multiline clamp",
            "The shaper truncates after two visual lines and places the ellipsis at the requested edge.",
            clamped_text(theme),
            theme,
        ),
        preview(
            "Document selection",
            "Drag across paragraphs, double-click a word, triple-click a line, or use Ctrl/Cmd+A and Ctrl/Cmd+C.",
            selection_examples(theme),
            theme,
        ),
    ])
    .gap(28.0)
}

fn rich_text(theme: &WidgetTheme) -> Element {
    let content = TextContent::rich([
        TextSpan::new("Argui ").style(TextSpanStyle::default().weight(760)),
        TextSpan::new("shapes rich spans ").style(
            TextSpanStyle::default()
                .color(theme.primary)
                .font_style(FontStyle::Italic),
        ),
        TextSpan::new("inside one Unicode-aware paragraph.").style(
            TextSpanStyle::default().decoration(TextDecoration {
                underline: UnderlineStyle::Single,
                underline_color: Some(theme.primary),
                ..TextDecoration::default()
            }),
        ),
    ]);
    Element::text(content)
        .text_style(TextStyle {
            font_size: 18.0,
            line_height: 28.0,
            color: theme.foreground,
            wrap: TextWrap::Word,
            ..TextStyle::default()
        })
        .width(percent(1.0))
}

fn type_styles(theme: &WidgetTheme) -> Element {
    let sample = |label, style| Element::text(label).text_style(style).width(percent(1.0));
    Element::column([
        sample(
            "WIDE TRACKING · GPU TYPOGRAPHY",
            TextStyle {
                font_size: 13.0,
                line_height: 22.0,
                color: theme.foreground,
                weight: 650,
                letter_spacing: LetterSpacing::Em(0.16),
                wrap: TextWrap::None,
                ..TextStyle::default()
            },
        ),
        sample(
            "Italic and expanded text remain shaped as text.",
            TextStyle {
                font_size: 17.0,
                line_height: 24.0,
                color: theme.foreground,
                font_style: FontStyle::Italic,
                stretch: FontStretch::Expanded,
                ..TextStyle::default()
            },
        ),
        sample(
            "Double underline and strikethrough can carry independent colors.",
            TextStyle {
                font_size: 16.0,
                line_height: 25.0,
                color: theme.foreground,
                decoration: TextDecoration {
                    underline: UnderlineStyle::Double,
                    underline_color: Some(theme.primary),
                    strikethrough: true,
                    strikethrough_color: Some(theme.destructive),
                },
                ..TextStyle::default()
            },
        ),
    ])
    .gap(10.0)
}

fn clamped_text(theme: &WidgetTheme) -> Element {
    Element::text(
        "A GPU interface still needs browser-grade prose. This intentionally long paragraph wraps naturally, remains selectable, and is clamped after exactly two visual lines without a separate clipping widget or pre-truncated string.",
    )
    .text_style(TextStyle {
        font_size: 15.0,
        line_height: 23.0,
        color: theme.foreground,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Ellipsis(EllipsisPosition::End),
        line_clamp: NonZeroUsize::new(2),
        ..TextStyle::default()
    })
    .width(percent(1.0))
    .max_width(length(560.0))
}

fn selection_examples(theme: &WidgetTheme) -> Element {
    let selection = TextSelectionStyle {
        background: with_alpha(theme.primary, 0.34),
        handle: theme.primary,
    };
    Element::column([
        Element::text(
            "This first paragraph and the next one form a single document selection, including mixed العربية and עברית text.",
        ),
        Element::text(
            "Selection follows glyph geometry, rounded clipping and transforms instead of using the element box.",
        ),
        Element::text("This note opts out through UserSelect::None.")
            .user_select(UserSelect::None)
            .text_style(TextStyle {
                color: theme.muted_foreground,
                ..body_style(theme)
            }),
    ])
    .gap(8.0)
    .padding(Sides::length(16.0))
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(10.0))
    .selection_style(selection)
    .width(percent(1.0))
}

fn with_alpha(color: argui::core::Color, alpha: f32) -> argui::core::Color {
    color.with_alpha(alpha)
}

fn body_style(theme: &WidgetTheme) -> TextStyle {
    TextStyle {
        font_size: 15.0,
        line_height: 23.0,
        color: theme.foreground,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    }
}
