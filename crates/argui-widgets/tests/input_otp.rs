use argui_core::{Color, ColorScheme};
use argui_ui::{Element, ElementKind, TextInputFilter, UiEvent, UiEventKind, UiTree};
use argui_widgets::{InputOtp, shadcn};

#[test]
fn otp_is_one_filtered_editor_and_completion_requires_exact_ascii_digits() {
    let mut otp = InputOtp::new("otp", "Verification code", "12a34567", 6);
    let element = otp.build(shadcn(Color::BLACK).resolve(ColorScheme::Light));
    let ElementKind::TextEditor { value, filter, .. } = &element.kind else {
        panic!("one real editor")
    };
    assert_eq!(value, "123456");
    assert_eq!(*filter, TextInputFilter::Digits { max_length: 6 });
    assert!(!otp.complete());
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for (value, valid) in [
        ("", true),
        ("123", true),
        ("123456", true),
        ("1234567", false),
        ("1a", false),
        ("１２３", false),
    ] {
        let event = UiEvent::new(
            node,
            Some("otp".into()),
            UiEventKind::TextChanged(value.into()),
        );
        assert_eq!(otp.action(&event), valid.then(|| value.to_owned()));
        otp.value = value.into();
        assert_eq!(otp.complete(), value == "123456");
    }
    assert!(
        otp.action(&UiEvent::new(
            node,
            Some("otp".into()),
            UiEventKind::Focused
        ))
        .is_none()
    );
    otp.enabled = false;
    assert!(
        otp.action(&UiEvent::new(
            node,
            Some("otp".into()),
            UiEventKind::TextChanged("1".into())
        ))
        .is_none()
    );
    otp.enabled = true;
    assert!(
        otp.action(&UiEvent::new(
            node,
            Some("other".into()),
            UiEventKind::TextChanged("1".into())
        ))
        .is_none()
    );
    assert_eq!(InputOtp::new("empty", "Code", "", 0).length, 1);
}
