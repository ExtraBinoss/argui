use argui_platform::WindowKey;
use argui_runtime::{Context, Render};
use argui_testing::{TestError, TestWindows};
use argui_ui::Element;

struct WindowContent;

impl Render for WindowContent {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("Window")
    }
}

#[test]
fn window_collection_reports_duplicate_and_missing_keys() {
    let mut windows = TestWindows::try_new(WindowContent).unwrap();
    assert_eq!(windows.entity().read(|_| 7), 7);
    let auxiliary = WindowKey::new("auxiliary-errors");
    windows.open(auxiliary.clone()).unwrap();

    let duplicate = match windows.open(auxiliary.clone()) {
        Ok(_) => panic!("duplicate window unexpectedly opened"),
        Err(error) => error,
    };
    assert!(matches!(duplicate, TestError::DuplicateWindow { .. }));
    windows.settle().unwrap();
    windows.close(&auxiliary).unwrap();
    let missing = match windows.window(&auxiliary) {
        Ok(_) => panic!("closed window unexpectedly found"),
        Err(error) => error,
    };
    assert!(matches!(missing, TestError::MissingWindow { .. }));
    assert!(matches!(
        windows.close(&auxiliary).unwrap_err(),
        TestError::MissingWindow { .. }
    ));
}
