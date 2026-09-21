#[cfg(feature = "icons")]
#[test]
fn every_pinned_icon_has_svg_and_importable_component_source() {
    let names = argui_dsl_stdlib::icon_names().collect::<Vec<_>>();
    assert_eq!(names.len(), 5945);
    assert!(names.contains(&"Spinner"));
    for name in names {
        let svg = argui_dsl_stdlib::icon_svg(name).expect("catalog icon has glyph data");
        assert!(svg.starts_with("<svg "), "{name}");
        assert!(svg.ends_with("</svg>"), "{name}");
        let source =
            argui_dsl_stdlib::icon_component_source(name).expect("catalog icon has import source");
        assert!(source.contains(&format!("export component {name}")));
    }
    assert_eq!(
        argui_dsl_stdlib::icon_svg("Spinner"),
        argui_dsl_stdlib::icon_svg("Loader2")
    );
    let spinner = argui_dsl_stdlib::icon_component_source("Spinner").unwrap();
    assert!(spinner.contains("in property rotation: float = 0.0"));
    assert!(spinner.contains("in property opacity: float = 1.0"));
    assert!(!spinner.contains("animate rotation"));
    assert!(
        argui_dsl_stdlib::icon_component_source("Star")
            .unwrap()
            .contains("in property rotation: float = 0.0")
    );
    assert!(argui_dsl_stdlib::icon_svg("NotATablerIcon").is_none());
}

#[cfg(not(feature = "icons"))]
#[test]
fn disabled_catalog_exposes_no_icons() {
    assert_eq!(argui_dsl_stdlib::icon_names().count(), 0);
    assert!(argui_dsl_stdlib::icon_svg("Star").is_none());
    assert!(argui_dsl_stdlib::icon_component_source("Star").is_none());
}
