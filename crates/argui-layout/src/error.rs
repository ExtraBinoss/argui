use std::{error::Error, fmt};

#[derive(Debug)]
pub enum LayoutError {
    Taffy(taffy::TaffyError),
    MissingRoot,
    MissingNodeIdentity(usize),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Taffy(error) => write!(formatter, "layout failed: {error}"),
            Self::MissingRoot => formatter.write_str("layout tree has no root"),
            Self::MissingNodeIdentity(index) => {
                write!(formatter, "UI node {index} has no stable identity")
            }
        }
    }
}

impl Error for LayoutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Taffy(error) => Some(error),
            Self::MissingRoot | Self::MissingNodeIdentity(_) => None,
        }
    }
}

impl From<taffy::TaffyError> for LayoutError {
    fn from(error: taffy::TaffyError) -> Self {
        Self::Taffy(error)
    }
}
