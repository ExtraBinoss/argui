use argui_media::VectorLibrary;

const ICON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="none" stroke="white" stroke-width="2" d="M9 6l6 6l-6 6"/></svg>"#;

#[test]
fn library_generates_handles_and_retains_assets() {
    let mut library = VectorLibrary::new();
    let first = library.insert_svg(ICON).unwrap();
    let second = library.insert_svg(ICON).unwrap();

    assert_ne!(first, second);
    assert_eq!(library.assets().len(), 2);
    assert_eq!(library.assets()[0].id, first);
    assert_eq!(library.assets()[1].id, second);
}
