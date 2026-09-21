//! Read-only, cross-platform terminal presentation for `argui dev`.

use std::{collections::VecDeque, io, io::IsTerminal, net::SocketAddr, path::Path};

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        ExecutableCommand,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use argui_dsl_protocol::LiveMessage;

use crate::SourceChange;

/// Amber accent used for pending generations and recoverable warnings.
pub const AMBER: Color = Color::Rgb(245, 158, 11);

/// Observable stage of a live update, rather than an estimated compiler percentage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DevPhase {
    Watching,
    Changed,
    Compiled,
    Applied,
    Rejected,
}

impl DevPhase {
    /// Returns the stage completion ratio displayed by the progress gauge.
    #[must_use]
    pub const fn ratio(self) -> f64 {
        match self {
            Self::Watching => 0.0,
            Self::Changed => 0.25,
            Self::Compiled => 0.65,
            Self::Applied | Self::Rejected => 1.0,
        }
    }

    /// Returns the stage label and its semantic display color.
    #[must_use]
    pub const fn presentation(self) -> (&'static str, Color) {
        match self {
            Self::Watching => ("WATCHING", Color::Cyan),
            Self::Changed => ("CHANGE DETECTED", AMBER),
            Self::Compiled => ("COMPILED · AWAITING CLIENT", AMBER),
            Self::Applied => ("APPLIED", Color::Green),
            Self::Rejected => ("REJECTED", Color::Red),
        }
    }
}

/// Bounded dashboard model, independent of terminal and input handling.
pub struct DevDashboard {
    root: String,
    native: SocketAddr,
    browser: SocketAddr,
    phase: DevPhase,
    generation: Option<u64>,
    native_clients: usize,
    events: VecDeque<(String, Color)>,
}

impl DevDashboard {
    /// Creates the dashboard for `root` and its native and browser addresses.
    #[must_use]
    pub fn new(root: &Path, native: SocketAddr, browser: SocketAddr) -> Self {
        Self {
            root: root.display().to_string(),
            native,
            browser,
            phase: DevPhase::Watching,
            generation: None,
            native_clients: 0,
            events: VecDeque::with_capacity(64),
        }
    }

    /// Returns the current observable update stage.
    #[must_use]
    pub const fn phase(&self) -> DevPhase {
        self.phase
    }

    /// Records one source delta with its exact known location and values.
    pub fn change(&mut self, change: &SourceChange) {
        self.phase = DevPhase::Changed;
        let location = match (change.line, change.column) {
            (Some(line), Some(column)) => format!("{}:{line}:{column}", change.path),
            _ => change.path.clone(),
        };
        self.push(format!("Change detected · {location}"), AMBER);
        self.push(format!("  − {}", change.before), Color::Red);
        self.push(format!("  + {}", change.after), Color::Green);
    }

    /// Records compilation success or diagnostics from one package generation.
    pub fn compilation(&mut self, message: &LiveMessage) {
        match message {
            LiveMessage::Package(package) => {
                self.phase = DevPhase::Compiled;
                self.generation = Some(package.header.generation);
                self.push(
                    format!(
                        "Generation {} compiled · awaiting client",
                        package.header.generation
                    ),
                    Color::Green,
                );
            }
            LiveMessage::Diagnostics {
                generation,
                diagnostics,
            } => {
                self.phase = DevPhase::Rejected;
                self.generation = Some(*generation);
                for diagnostic in diagnostics {
                    self.push(
                        format!(
                            "Generation {generation} rejected · {}: {}",
                            diagnostic.code, diagnostic.message
                        ),
                        Color::Red,
                    );
                }
            }
            _ => {}
        }
    }

    /// Records the count of native clients that remain connected.
    pub fn clients(&mut self, count: usize) {
        self.native_clients = count;
    }

    /// Records one native client connection.
    pub fn connected(&mut self, peer: SocketAddr) {
        self.push(format!("Native client connected · {peer}"), Color::Cyan);
    }

    /// Records one runtime acknowledgement, rejection, or restart requirement.
    pub fn client_status(&mut self, peer: SocketAddr, message: &LiveMessage) {
        let acknowledged_generation = match message {
            LiveMessage::Committed { generation }
            | LiveMessage::Rejected { generation, .. }
            | LiveMessage::RestartRequired { generation, .. } => Some(*generation),
            _ => None,
        };
        if let Some((acknowledged, current)) = acknowledged_generation.zip(self.generation)
            && acknowledged < current
        {
            self.push(
                format!("Delayed generation {acknowledged} status from {peer} · current generation {current}"),
                Color::DarkGray,
            );
            return;
        }
        match message {
            LiveMessage::Committed { generation } => {
                self.phase = DevPhase::Applied;
                self.generation = Some(*generation);
                self.push(
                    format!("Generation {generation} applied by {peer}"),
                    Color::Green,
                );
            }
            LiveMessage::Rejected {
                generation,
                message,
            } => {
                self.phase = DevPhase::Rejected;
                self.generation = Some(*generation);
                self.push(
                    format!("Generation {generation} rejected by {peer} · {message}"),
                    Color::Red,
                );
            }
            LiveMessage::RestartRequired { generation, .. } => {
                self.phase = DevPhase::Rejected;
                self.generation = Some(*generation);
                self.push(
                    format!("Generation {generation} changes Rust ABI · restart the app"),
                    AMBER,
                );
            }
            _ => self.push(format!("Unexpected client status from {peer}"), AMBER),
        }
    }

    /// Appends a bounded diagnostic or application-output line with the given color.
    pub fn note(&mut self, message: impl Into<String>, color: Color) {
        self.push(message.into(), color);
    }

    /// Renders one frame without polling input or scheduling idle redraws.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(2),
            ])
            .split(area);
        let heading = Line::from(vec![
            Span::styled(
                "◆ ARGUI ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "DEV",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   {}", self.root)),
        ]);
        frame.render_widget(
            Paragraph::new(heading).block(Block::default().borders(Borders::ALL)),
            sections[0],
        );
        let (phase_label, phase_color) = self.phase.presentation();
        frame.render_widget(
            Gauge::default()
                .block(
                    Block::default()
                        .title(" Live update ")
                        .borders(Borders::ALL),
                )
                .gauge_style(Style::default().fg(phase_color))
                .label(format!(
                    "{phase_label} · generation {}",
                    self.generation
                        .map_or_else(|| "—".into(), |value| value.to_string())
                ))
                .ratio(self.phase.ratio()),
            sections[1],
        );
        let connection_color = if self.native_clients == 0 {
            AMBER
        } else {
            Color::Green
        };
        let connections = Line::from(vec![
            Span::styled(
                format!("● {} native  ", self.native_clients),
                Style::default().fg(connection_color),
            ),
            Span::raw(format!("tcp://{}   ws://{}", self.native, self.browser)),
        ]);
        frame.render_widget(
            Paragraph::new(connections).block(
                Block::default()
                    .title(" Connections ")
                    .borders(Borders::ALL),
            ),
            sections[2],
        );
        let visible = usize::from(sections[3].height.saturating_sub(2));
        let lines = self
            .events
            .iter()
            .rev()
            .take(visible)
            .rev()
            .map(|(message, color)| Line::styled(message.as_str(), Style::default().fg(*color)))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(lines).block(Block::default().title(" Activity ").borders(Borders::ALL)),
            sections[3],
        );
        frame.render_widget(
            Paragraph::new("Ctrl+C to stop · no mouse input")
                .style(Style::default().fg(Color::DarkGray)),
            sections[4],
        );
    }

    /// Adds an event and drops the oldest one when the history is full.
    fn push(&mut self, message: String, color: Color) {
        if self.events.len() == 64 {
            self.events.pop_front();
        }
        self.events.push_back((message, color));
    }
}

/// Optional terminal renderer; redirected runs retain ordinary line-oriented output.
pub struct DevConsole {
    dashboard: DevDashboard,
    terminal: Option<Terminal<CrosstermBackend<io::Stderr>>>,
}

impl DevConsole {
    /// Enters the alternate screen only for an interactive terminal.
    ///
    /// # Errors
    ///
    /// Returns a terminal setup error if the alternate screen cannot be initialized.
    pub fn new(root: &Path, native: SocketAddr, browser: SocketAddr) -> io::Result<Self> {
        let interactive = io::stdin().is_terminal()
            && io::stderr().is_terminal()
            && std::env::var("TERM").map_or(true, |term| term != "dumb");
        let terminal = if interactive {
            let mut output = io::stderr();
            output.execute(EnterAlternateScreen)?;
            match Terminal::new(CrosstermBackend::new(output)) {
                Ok(terminal) => Some(terminal),
                Err(error) => {
                    let _ = io::stderr().execute(LeaveAlternateScreen);
                    return Err(error);
                }
            }
        } else {
            None
        };
        let mut console = Self {
            dashboard: DevDashboard::new(root, native, browser),
            terminal,
        };
        console.draw()?;
        Ok(console)
    }

    /// Returns whether the alternate-screen dashboard is active.
    #[must_use]
    pub const fn interactive(&self) -> bool {
        self.terminal.is_some()
    }

    /// Applies a change to the dashboard and redraws it once.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn change(&mut self, change: &SourceChange) -> io::Result<()> {
        let location = match (change.line, change.column) {
            (Some(line), Some(column)) => format!("{}:{line}:{column}", change.path),
            _ => change.path.clone(),
        };
        let line = format!(
            "\n  ↻ Change detected · {location}\n    − {}\n    + {}",
            change.before, change.after
        );
        self.dispatch([line], |dashboard| dashboard.change(change))
    }

    /// Records one compilation result and redraws it once.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn compilation(&mut self, message: &LiveMessage) -> io::Result<()> {
        let lines = match message {
            LiveMessage::Package(package) => vec![format!(
                "  ✓ Generation {} compiled · awaiting client confirmation",
                package.header.generation
            )],
            LiveMessage::Diagnostics {
                generation,
                diagnostics,
            } => diagnostics
                .iter()
                .map(|diagnostic| {
                    format!(
                        "  ✗ Generation {generation} rejected · {}: {}",
                        diagnostic.code, diagnostic.message
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
        self.dispatch(lines, |dashboard| dashboard.compilation(message))
    }

    /// Records a client acknowledgement and redraws the terminal.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn client_status(&mut self, peer: SocketAddr, message: &LiveMessage) -> io::Result<()> {
        let lines = match message {
            LiveMessage::Committed { generation } => {
                vec![format!("  ✓ Generation {generation} applied by {peer}")]
            }
            LiveMessage::Rejected {
                generation,
                message,
            } => vec![format!(
                "  ✗ Generation {generation} rejected by {peer} · {message}"
            )],
            LiveMessage::RestartRequired { generation, .. } => vec![format!(
                "  ⚠ Generation {generation} changes the Rust-facing ABI · restart the app"
            )],
            _ => vec![format!("  ⚠ Unexpected client status from {peer}")],
        };
        self.dispatch(lines, |dashboard| dashboard.client_status(peer, message))
    }

    /// Records an informational line and redraws the terminal.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn note(&mut self, message: impl Into<String>, color: Color) -> io::Result<()> {
        let message = message.into();
        self.dispatch([format!("  {message}")], |dashboard| {
            dashboard.note(message, color);
        })
    }

    /// Records a burst of application output and performs at most one redraw.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn notes(
        &mut self,
        messages: impl IntoIterator<Item = String>,
        color: Color,
    ) -> io::Result<()> {
        let messages = messages.into_iter().collect::<Vec<_>>();
        if messages.is_empty() {
            return Ok(());
        }
        let lines = messages
            .iter()
            .map(|message| format!("  {message}"))
            .collect::<Vec<_>>();
        self.dispatch(lines, |dashboard| {
            for message in messages {
                dashboard.note(message, color);
            }
        })
    }

    /// Updates the connected native-client count and redraws it once.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn clients(&mut self, count: usize) -> io::Result<()> {
        if self.dashboard.native_clients == count {
            return Ok(());
        }
        self.dashboard.clients(count);
        self.draw()
    }

    /// Records a new native connection and redraws the terminal.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn connected(&mut self, peer: SocketAddr) -> io::Result<()> {
        self.dispatch(
            [format!("  ◇ Native client connected · {peer}")],
            |dashboard| dashboard.connected(peer),
        )
    }

    /// Emits non-interactive lines, applies one dashboard mutation, and redraws once.
    ///
    /// * `lines` — complete line-oriented messages for redirected console output.
    /// * `mutate` — dashboard update to apply after any redirected output.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors from the optional redraw.
    fn dispatch<I, F>(&mut self, lines: I, mutate: F) -> io::Result<()>
    where
        I: IntoIterator<Item = String>,
        F: FnOnce(&mut DevDashboard),
    {
        let lines = lines.into_iter().collect::<Vec<_>>();
        if !self.interactive() {
            for line in &lines {
                eprintln!("{line}");
            }
        }
        mutate(&mut self.dashboard);
        self.draw()
    }

    /// Redraws after an event, doing no work while idle.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    fn draw(&mut self) -> io::Result<()> {
        if let Some(terminal) = &mut self.terminal {
            terminal.draw(|frame| self.dashboard.render(frame))?;
        }
        Ok(())
    }
}

impl Drop for DevConsole {
    fn drop(&mut self) {
        if let Some(terminal) = &mut self.terminal {
            let _ = terminal.show_cursor();
            let _ = io::stderr().execute(LeaveAlternateScreen);
        }
    }
}
