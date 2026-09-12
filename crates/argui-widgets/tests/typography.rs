use argui_core::{Color, ColorScheme};
use argui_ui::{ElementKind, Role};
use argui_widgets::{Typography, TypographyVariant, shadcn};

#[test]
fn typography_has_theme_contrast_and_clamped_semantic_headings() {
    let palette = shadcn(Color::BLACK);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = palette.resolve(scheme);
        for variant in [
            TypographyVariant::Heading(0),
            TypographyVariant::Heading(2),
            TypographyVariant::Heading(3),
            TypographyVariant::Heading(7),
            TypographyVariant::Paragraph,
            TypographyVariant::Lead,
            TypographyVariant::Large,
            TypographyVariant::Small,
            TypographyVariant::Muted,
            TypographyVariant::Code,
            TypographyVariant::Quote,
        ] {
            let element = Typography::new("Readable text", variant).build(theme);
            let ElementKind::Text { style, .. } = &element.kind else {
                panic!("text expected")
            };
            assert!(style.color.contrast_ratio(theme.background) >= 4.0);
            let semantics = element.semantics.as_ref().unwrap();
            if let TypographyVariant::Heading(level) = variant {
                assert_eq!(semantics.role, Role::Heading);
                assert_eq!(semantics.level, Some(level.clamp(1, 6)));
            }
        }
    }
}
