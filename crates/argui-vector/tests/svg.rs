use argui_paint::VectorId;
use argui_vector::{VectorError, morph_svg, parse_svg};

const RIGHT: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M9 6l6 6l-6 6"/></svg>"#;
const DOWN: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="M6 9l6 6l6 -6"/></svg>"#;

#[test]
fn tabler_svg_is_tessellated_without_pixels() {
    let asset = parse_svg(VectorId(1), RIGHT).unwrap();
    assert_eq!(asset.size.width, 24.0);
    assert!(!asset.vertices.is_empty());
    assert!(!asset.indices.is_empty());
}

#[test]
fn compatible_paths_morph_with_one_gpu_topology() {
    let asset = morph_svg(VectorId(2), RIGHT, DOWN).unwrap();
    assert!(asset.vertices.iter().any(|vertex| vertex.from != vertex.to));
}

#[test]
fn invalid_and_non_path_svg_are_rejected() {
    assert!(parse_svg(VectorId(3), b"nope").is_err());
    let image = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><image href="data:image/png;base64,"/></svg>"#;
    assert!(matches!(
        parse_svg(VectorId(4), image),
        Err(VectorError::UnsupportedNode) | Err(VectorError::Svg(_))
    ));
}

#[test]
fn svg_colors_are_linear_and_inherit_group_opacity() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
        <g opacity="0.5"><path fill="#808080" d="M0 0H10V10H0Z"/></g>
    </svg>"##;
    let asset = parse_svg(VectorId(5), svg).unwrap();
    let color = asset.vertices[0].color_from.as_array();
    assert!((color[0] - 0.216).abs() < 0.002);
    assert_eq!(color[3], 0.5);
}

#[test]
fn curves_subpaths_and_stroke_variants_share_the_vector_path() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32">
      <path display="none" d="M0 0H2V2Z"/>
      <path fill="#000000" stroke="#ffffff" stroke-linecap="square"
        stroke-linejoin="bevel" d="M1 1 Q8 20 15 1 M17 1 C20 8 24 8 31 1"/>
      <path fill="none" stroke="#ffffff" stroke-linecap="butt"
        stroke-linejoin="miter" d="M1 30L16 18L31 30"/>
    </svg>"##;
    let asset = parse_svg(VectorId(6), svg).unwrap();
    assert!(!asset.indices.is_empty());
    assert_eq!(asset.vertices[0].color_from.as_array()[0], 0.0);
}

#[test]
fn gradients_and_incompatible_morphs_fail_explicitly() {
    let gradient = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
      <defs><linearGradient id="g"><stop stop-color="red"/><stop offset="1" stop-color="blue"/></linearGradient></defs>
      <path fill="url(#g)" d="M0 0H10V10H0Z"/>
    </svg>"##;
    assert!(matches!(
        parse_svg(VectorId(7), gradient),
        Err(VectorError::UnsupportedPaint)
    ));

    let square = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path d="M0 0H10V10H0Z"/></svg>"#;
    let triangle = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path d="M0 0L10 10H0Z"/></svg>"#;
    let larger = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><path d="M0 0H10V10H0Z"/></svg>"#;
    assert!(matches!(
        morph_svg(VectorId(8), square, triangle),
        Err(VectorError::IncompatibleMorph)
    ));
    assert!(matches!(
        morph_svg(VectorId(9), square, larger),
        Err(VectorError::IncompatibleMorph)
    ));
}

#[test]
fn fill_rules_paint_order_and_unsupported_svg_features_are_explicit() {
    let ordered = br##"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12">
      <path fill="#000" fill-rule="evenodd" stroke="#fff" stroke-width="2"
        paint-order="stroke fill" d="M1 1H11V11H1Z M4 4H8V8H4Z"/>
    </svg>"##;
    let asset = parse_svg(VectorId(10), ordered).unwrap();
    assert_eq!(asset.vertices[0].color_from.as_array()[0], 1.0);
    assert_eq!(asset.vertices.last().unwrap().color_from.as_array()[0], 0.0);

    let dashed = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><path fill="none" stroke="red" stroke-dasharray="2 2" d="M0 5H10"/></svg>"#;
    assert!(matches!(
        parse_svg(VectorId(11), dashed),
        Err(VectorError::UnsupportedFeature("dashed stroke"))
    ));

    let clipped = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><clipPath id="c"><path d="M0 0H5V5H0Z"/></clipPath><g clip-path="url(#c)"><path d="M0 0H10V10H0Z"/></g></svg>"#;
    assert!(matches!(
        parse_svg(VectorId(12), clipped),
        Err(VectorError::UnsupportedFeature("group clip/mask/filter"))
    ));
}
