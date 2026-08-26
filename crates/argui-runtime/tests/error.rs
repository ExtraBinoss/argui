use std::error::Error;

use argui_layout::LayoutError;
use argui_platform::PlatformError;
use argui_render::RendererError;
use argui_runtime::RuntimeError;
use winit::error::EventLoopError;

#[test]
fn runtime_error_keeps_the_renderer_cause() {
    let error = RuntimeError::from(RendererError::Validation);

    assert_eq!(error.to_string(), "surface validation failed");
    assert!(error.source().is_some());

    let error = RuntimeError::from(PlatformError::from(EventLoopError::ExitFailure(4)));
    assert_eq!(error.to_string(), "platform error: Exit Failure: 4");
    assert!(error.source().is_some());
}

#[test]
fn runtime_error_keeps_the_layout_cause() {
    let error = RuntimeError::from(LayoutError::MissingRoot);

    assert_eq!(error.to_string(), "layout tree has no root");
    assert!(std::error::Error::source(&error).is_some());
}
