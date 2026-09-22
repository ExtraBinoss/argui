//! Read-only, cross-platform terminal presentation for `argui dev`.

use std::{collections::VecDeque, net::SocketAddr, path::Path, time::Instant};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use argui_dsl_protocol::{DiagnosticMessage, LiveMessage, Severity};

use crate::SourceChange;

/// Amber accent used for pending generations and recoverable warnings.
pub const AMBER: Color = Color::Rgb(245, 158, 11);

mod console;
mod progress;
pub use console::DevConsole;
use progress::ProgressBar;

/// Observable stage of a live update, rather than an estimated compiler percentage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DevPhase {
    Watching,
    Changed,
    Compiled,
    BuildingClient,
    Applied,
    Rejected,
}

impl DevPhase {
    /// Returns the stage completion ratio displayed by the progress bar.
    #[must_use]
    pub const fn ratio(self) -> f64 {
        match self {
            Self::Watching => 0.0,
            Self::Changed => 0.25,
            Self::Compiled => 0.65,
            Self::BuildingClient => 0.0,
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
            Self::BuildingClient => ("BUILDING NATIVE CLIENT", AMBER),
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
    status: String,
    native_build: Option<NativeBuild>,
    events: VecDeque<(String, Color)>,
}

/// Observed Cargo progress for the native client build.
struct NativeBuild {
    started: Instant,
    completed: usize,
    total: usize,
    crate_name: String,
    shown_second: u64,
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
            status: "Waiting for source changes".into(),
            native_build: None,
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
        self.status = format!("Compiling change in {}", change.path);
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
                if self.native_build.is_none() {
                    self.phase = DevPhase::Compiled;
                    self.status = "Package ready; waiting for a connected client".into();
                }
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
                self.status = diagnostics
                    .iter()
                    .find(|diagnostic| diagnostic.severity == Severity::Error)
                    .or_else(|| diagnostics.first())
                    .map_or_else(
                        || "Compilation rejected without details".into(),
                        |diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message),
                    );
                for diagnostic in diagnostics {
                    self.push(
                        format!(
                            "Generation {generation} rejected · {} · {}: {}",
                            self.diagnostic_location(diagnostic),
                            diagnostic.code,
                            diagnostic.message
                        ),
                        Color::Red,
                    );
                }
            }
            _ => {}
        }
    }

    /// Starts a separately observable Cargo build for the native live client.
    pub(crate) fn native_build_started(&mut self) {
        self.phase = DevPhase::BuildingClient;
        self.native_build = Some(NativeBuild {
            started: Instant::now(),
            completed: 0,
            total: 0,
            crate_name: "starting Cargo".into(),
            shown_second: 0,
        });
        self.refresh_native_build_status();
        self.push("Building native client with Cargo".into(), AMBER);
    }

    /// Records a Cargo compilation target and any exact completed/total step count.
    ///
    /// `crate_name` is Cargo's current target; `steps` is present for a progress frame.
    pub(crate) fn native_build_progress(
        &mut self,
        crate_name: &str,
        steps: Option<(usize, usize)>,
    ) {
        if let Some(build) = &mut self.native_build {
            build.crate_name = crate_name.to_owned();
            if let Some((completed, total)) = steps {
                build.completed = completed;
                build.total = total;
            }
            self.refresh_native_build_status();
        }
    }

    /// Updates the build elapsed time, returning whether the dashboard needs a redraw.
    pub(crate) fn native_build_tick(&mut self) -> bool {
        let Some(build) = &mut self.native_build else {
            return false;
        };
        let second = build.started.elapsed().as_secs();
        if second == build.shown_second {
            return false;
        }
        build.shown_second = second;
        self.refresh_native_build_status();
        true
    }

    /// Marks Cargo's handoff to the running native application.
    pub(crate) fn native_build_finished(&mut self) {
        if self.native_build.take().is_some() {
            self.phase = DevPhase::Compiled;
            self.status = "Native client started; waiting for its connection".into();
            self.push("Native client started".into(), Color::Green);
        }
    }

    /// Formats the current Cargo count, target and elapsed time in the status line.
    fn refresh_native_build_status(&mut self) {
        if let Some(build) = &self.native_build {
            let count = if build.total > 0 {
                format!("{}/{} · ", build.completed, build.total)
            } else {
                String::new()
            };
            self.status = format!(
                "Cargo {count}{} · {}s",
                build.crate_name,
                build.started.elapsed().as_secs()
            );
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
                self.native_build = None;
                self.phase = DevPhase::Applied;
                self.generation = Some(*generation);
                self.status = format!("Generation {generation} confirmed by {peer}");
                self.push(
                    format!("Generation {generation} applied by {peer}"),
                    Color::Green,
                );
            }
            LiveMessage::Rejected {
                generation,
                message,
            } => {
                self.native_build = None;
                self.phase = DevPhase::Rejected;
                self.generation = Some(*generation);
                self.status = message.clone();
                self.push(
                    format!("Generation {generation} rejected by {peer} · {message}"),
                    Color::Red,
                );
            }
            LiveMessage::RestartRequired { generation, .. } => {
                self.native_build = None;
                self.phase = DevPhase::Rejected;
                self.generation = Some(*generation);
                self.status = "Rust API changed; restart the application".into();
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

    /// Renders a full or compact frame according to the current terminal dimensions.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        if area.width < 28 || area.height < 7 {
            frame.render_widget(
                Paragraph::new("Argui dev: enlarge the terminal")
                    .style(Style::default().fg(AMBER))
                    .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }
        let compact = area.width < 68 || area.height < 13;
        let sections = if compact {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Min(1),
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(area)
        };
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
            Span::raw(if compact {
                String::new()
            } else {
                format!("   {}", self.root)
            }),
        ]);
        let heading = Paragraph::new(heading);
        if compact {
            frame.render_widget(heading, sections[0]);
        } else {
            frame.render_widget(
                heading.block(Block::default().borders(Borders::ALL)),
                sections[0],
            );
        }
        let (phase_label, phase_color) = self.phase.presentation();
        let generation = self
            .generation
            .map_or_else(|| "—".into(), |value| value.to_string());
        let progress = ProgressBar {
            ratio: self.native_build.as_ref().map_or_else(
                || self.phase.ratio(),
                |build| {
                    if build.total == 0 {
                        0.0
                    } else {
                        build.completed as f64 / build.total as f64
                    }
                },
            ),
            color: phase_color,
        };
        progress.render(frame, sections[1], phase_label, &generation, &self.status);
        let activity = if compact {
            sections[2]
        } else {
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
            sections[3]
        };
        let lines = self
            .events
            .iter()
            .map(|(message, color)| Line::styled(message.as_str(), Style::default().fg(*color)))
            .collect::<Vec<_>>();
        let visible = usize::from(activity.height.saturating_sub(2));
        let content_width = usize::from(activity.width.saturating_sub(2)).max(1);
        let rows = self
            .events
            .iter()
            .map(|(message, _)| message.chars().count().div_ceil(content_width).max(1))
            .sum::<usize>();
        let scroll = rows.saturating_sub(visible);
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .title(" Activity · Ctrl+C to stop ")
                .borders(Borders::ALL),
        );
        frame.render_widget(
            paragraph.scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
            activity,
        );
    }

    /// Resolves a diagnostic byte offset to a one-based source line and column.
    #[must_use]
    pub fn diagnostic_location(&self, diagnostic: &DiagnosticMessage) -> String {
        let Some(path) = diagnostic.path.as_deref() else {
            return "source location unavailable".into();
        };
        let Some(offset) = diagnostic.start else {
            return path.into();
        };
        let source = std::fs::read_to_string(Path::new(&self.root).join(path));
        let Ok(source) = source else {
            return format!("{path}:byte {offset}");
        };
        let end = usize::try_from(offset)
            .unwrap_or(usize::MAX)
            .min(source.len());
        let boundary = (0..=end)
            .rev()
            .find(|index| source.is_char_boundary(*index))
            .unwrap_or(0);
        let prefix = &source[..boundary];
        let line = prefix
            .chars()
            .filter(|character| *character == '\n')
            .count()
            + 1;
        let column = prefix
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .count()
            + 1;
        format!("{path}:{line}:{column}")
    }

    /// Adds an event and drops the oldest one when the history is full.
    fn push(&mut self, message: String, color: Color) {
        if self.events.len() == 64 {
            self.events.pop_front();
        }
        self.events.push_back((message, color));
    }
}
