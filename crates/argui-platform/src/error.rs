use std::{error::Error, fmt};

#[derive(Debug)]
pub struct PlatformError(winit::error::EventLoopError);

impl fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "platform error: {}", self.0)
    }
}

impl Error for PlatformError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<winit::error::EventLoopError> for PlatformError {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self(error)
    }
}

impl From<winit::error::OsError> for PlatformError {
    fn from(error: winit::error::OsError) -> Self {
        Self(error.into())
    }
}
