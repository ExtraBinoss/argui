use std::{error::Error, fmt};

#[derive(Debug)]
pub enum LayoutError {
    Custom(String),
    InvalidBoundary(&'static str),
    Taffy(taffy::TaffyError),
    MissingRoot,
    MissingNodeIdentity(usize),
    StaleOutput,
    NonConvergentContainerQueries,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(error) => write!(formatter, "custom element: {error}"),
            Self::InvalidBoundary(error) => write!(formatter, "layout boundary: {error}"),
            Self::Taffy(error) => write!(formatter, "layout failed: {error}"),
            Self::MissingRoot => formatter.write_str("layout tree has no root"),
            Self::MissingNodeIdentity(index) => {
                write!(formatter, "UI node {index} has no stable identity")
            }
            Self::StaleOutput => formatter.write_str("scroll layout belongs to an older UI tree"),
            Self::NonConvergentContainerQueries => {
                formatter.write_str("container queries did not converge after four layout passes")
            }
        }
    }
}

impl Error for LayoutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Taffy(error) => Some(error),
            Self::Custom(_)
            | Self::InvalidBoundary(_)
            | Self::MissingRoot
            | Self::MissingNodeIdentity(_)
            | Self::StaleOutput
            | Self::NonConvergentContainerQueries => None,
        }
    }
}

impl From<taffy::TaffyError> for LayoutError {
    fn from(error: taffy::TaffyError) -> Self {
        Self::Taffy(error)
    }
}
