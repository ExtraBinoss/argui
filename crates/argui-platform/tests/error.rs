use std::error::Error;

use argui_platform::PlatformError;
use winit::error::EventLoopError;

#[test]
fn platform_error_preserves_its_winit_cause() {
    let error = PlatformError::from(EventLoopError::ExitFailure(7));

    assert_eq!(error.to_string(), "platform error: Exit Failure: 7");
    assert!(error.source().is_some());
}
