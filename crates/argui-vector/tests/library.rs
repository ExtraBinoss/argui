use argui_vector::VectorLibrary;

const RIGHT: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="none" stroke="white" stroke-width="2" d="M9 6l6 6l-6 6"/></svg>"#;
const DOWN: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="none" stroke="white" stroke-width="2" d="M6 9l6 6l6 -6"/></svg>"#;

#[test]
fn library_generates_handles_and_retains_plain_and_morph_assets() {
    let mut library = VectorLibrary::new();
    let icon = library.insert_svg(RIGHT).unwrap();
    let morph = library.insert_morph(RIGHT, DOWN).unwrap();

    assert_ne!(icon, morph);
    assert_eq!(library.assets().len(), 2);
    assert_eq!(library.assets()[0].id, icon);
    assert_eq!(library.assets()[1].id, morph);
}
