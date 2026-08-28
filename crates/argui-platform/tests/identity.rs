use argui_platform::{AppIcon, ApplicationId, ApplicationIdentity, IconSet};

#[test]
fn application_ids_are_strict_reverse_dns_names() {
    assert!(ApplicationId::new("dev.argui.state").is_ok());
    for invalid in [
        "argui",
        "Dev.argui",
        "dev..argui",
        "1dev.argui",
        "dev.arg_ui",
        "dev.argui.Bad",
    ] {
        assert!(ApplicationId::new(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn linux_application_id_defaults_to_bundle_id_and_can_match_a_packager() {
    let identity = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.state").unwrap(),
        "State",
        IconSet::new(),
    );
    assert_eq!(identity.linux_application_id(), "dev.argui.state");
    assert_eq!(
        identity
            .with_linux_application_id("argui-state")
            .linux_application_id(),
        "argui-state"
    );
}

#[test]
fn rgba_icons_validate_and_encode_once() {
    let icon = AppIcon::from_rgba8(2, 2, vec![255; 16]).unwrap();
    assert_eq!((icon.width, icon.height), (2, 2));
    assert!(icon.png.starts_with(&[137, 80, 78, 71]));
    assert!(AppIcon::from_rgba8(0, 2, Vec::new()).is_err());
    assert!(AppIcon::from_rgba8(2, 2, vec![0; 15]).is_err());
}

#[test]
fn icon_sets_replace_duplicate_sizes_and_choose_the_closest() {
    let small = AppIcon::from_rgba8(16, 16, vec![0; 16 * 16 * 4]).unwrap();
    let large = AppIcon::from_rgba8(64, 64, vec![0; 64 * 64 * 4]).unwrap();
    let replacement = AppIcon::from_rgba8(16, 16, vec![1; 16 * 16 * 4]).unwrap();
    let icons = IconSet::new().with(small).with(large).with(replacement);
    assert_eq!(icons.icons().len(), 2);
    assert_eq!(icons.best_square(24).unwrap().width, 16);
    assert_eq!(icons.best_square(48).unwrap().width, 64);
    assert!(!icons.is_empty());
    assert!(IconSet::new().is_empty());
}

#[test]
fn png_icons_decode_and_reject_other_encoded_data() {
    let encoded = AppIcon::from_rgba8(1, 1, vec![12, 34, 56, 255])
        .unwrap()
        .png;
    let decoded = AppIcon::from_png(&encoded).unwrap();
    assert_eq!(&*decoded.rgba8, &[12, 34, 56, 255]);
    assert!(AppIcon::from_png(b"not a png").is_err());
}
