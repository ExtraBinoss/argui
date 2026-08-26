use crate::Quad;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayCommand {
    Quad(Quad),
    Text(usize),
}

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
}
