use argui_paint::VectorId;
use argui_vector::parse_svg;

const ICON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="none" stroke="currentColor" stroke-width="2" d="M9 6l6 6l-6 6"/></svg>"#;

#[test]
fn svg_retains_resolution_independent_source() {
    let asset = parse_svg(VectorId(1), ICON).unwrap();
    assert_eq!(asset.size.width, 24.0);
    assert_eq!(asset.size.height, 24.0);
    assert_eq!(&*asset.svg, ICON);
    assert!(asset.tintable);
}

#[test]
fn invalid_svg_is_rejected() {
    assert!(parse_svg(VectorId(2), b"nope").is_err());
}

#[test]
fn complete_static_features_are_retained() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12">
      <defs>
        <linearGradient id="g"><stop stop-color="red"/><stop offset="1" stop-color="blue"/></linearGradient>
        <clipPath id="c"><path d="M0 0H8V8H0Z"/></clipPath>
      </defs>
      <g clip-path="url(#c)"><path fill="url(#g)" stroke="white" stroke-dasharray="2 2" d="M0 0H12V12H0Z"/></g>
    </svg>"##;
    let asset = parse_svg(VectorId(3), svg).unwrap();
    assert_eq!(&*asset.svg, svg);
    assert!(!asset.tintable);
}

#[test]
fn current_color_detection_is_case_insensitive() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><path fill="CURRENTCOLOR" d="M0 0H1V1Z"/></svg>"#;
    assert!(parse_svg(VectorId(4), svg).unwrap().tintable);
}
