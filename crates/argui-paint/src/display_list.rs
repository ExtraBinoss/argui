use core::fmt;

use crate::{LayerStyle, Quad};

#[derive(Clone, Debug, PartialEq)]
pub enum DisplayCommand {
    Quad(Quad),
    Text(usize),
    BeginLayer(LayerStyle),
    EndLayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayListError {
    UnexpectedLayerEnd { command: usize },
    UnclosedLayers { count: usize },
}

impl fmt::Display for DisplayListError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedLayerEnd { command } => {
                write!(
                    formatter,
                    "layer ends without a matching begin at command {command}"
                )
            }
            Self::UnclosedLayers { count } => {
                write!(formatter, "display list has {count} unclosed layers")
            }
        }
    }
}

impl std::error::Error for DisplayListError {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DisplayList {
    commands: Vec<DisplayCommand>,
    quad_count: usize,
}

impl DisplayList {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
            quad_count: 0,
        }
    }

    pub fn push_quad(&mut self, quad: Quad) {
        self.commands.push(DisplayCommand::Quad(quad));
        self.quad_count += 1;
    }

    pub fn push_text(&mut self, block: usize) {
        self.commands.push(DisplayCommand::Text(block));
    }

    pub fn begin_layer(&mut self, style: LayerStyle) {
        self.commands.push(DisplayCommand::BeginLayer(style));
    }

    pub fn end_layer(&mut self) {
        self.commands.push(DisplayCommand::EndLayer);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.quad_count = 0;
    }

    #[must_use]
    pub fn commands(&self) -> &[DisplayCommand] {
        &self.commands
    }

    #[must_use]
    pub const fn quad_count(&self) -> usize {
        self.quad_count
    }

    pub fn validate(&self) -> Result<(), DisplayListError> {
        let mut depth = 0_usize;
        for (command, item) in self.commands.iter().enumerate() {
            match item {
                DisplayCommand::BeginLayer(_) => depth += 1,
                DisplayCommand::EndLayer if depth == 0 => {
                    return Err(DisplayListError::UnexpectedLayerEnd { command });
                }
                DisplayCommand::EndLayer => depth -= 1,
                DisplayCommand::Quad(_) | DisplayCommand::Text(_) => {}
            }
        }
        if depth == 0 {
            Ok(())
        } else {
            Err(DisplayListError::UnclosedLayers { count: depth })
        }
    }
}
