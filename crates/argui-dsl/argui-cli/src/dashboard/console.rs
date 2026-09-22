//! Terminal lifecycle and event handling for the dev dashboard.

use std::{io, io::IsTerminal, net::SocketAddr, path::Path};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        ExecutableCommand,
        event::{self, DisableMouseCapture, Event, KeyCode, KeyModifiers},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    style::Color,
};

use argui_dsl_protocol::LiveMessage;

use crate::SourceChange;

use super::DevDashboard;

/// Optional terminal renderer; redirected runs retain ordinary line-oriented output.
pub struct DevConsole {
    dashboard: DevDashboard,
    terminal: Option<Terminal<CrosstermBackend<io::Stderr>>>,
    owns_raw_mode: bool,
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
        let owns_raw_mode = interactive && !ratatui::crossterm::terminal::is_raw_mode_enabled()?;
        let terminal = if interactive {
            if owns_raw_mode {
                enable_raw_mode()?;
            }
            let mut output = io::stderr();
            if let Err(error) = output
                .execute(DisableMouseCapture)
                .and_then(|output| output.execute(EnterAlternateScreen))
            {
                if owns_raw_mode {
                    let _ = disable_raw_mode();
                }
                return Err(error);
            }
            match Terminal::new(CrosstermBackend::new(output)) {
                Ok(terminal) => Some(terminal),
                Err(error) => {
                    let _ = io::stderr().execute(LeaveAlternateScreen);
                    if owns_raw_mode {
                        let _ = disable_raw_mode();
                    }
                    return Err(error);
                }
            }
        } else {
            None
        };
        let mut console = Self {
            dashboard: DevDashboard::new(root, native, browser),
            terminal,
            owns_raw_mode,
        };
        let _ = console.pump_input()?;
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
                        "  ✗ Generation {generation} rejected · {} · {}: {}",
                        self.dashboard.diagnostic_location(diagnostic),
                        diagnostic.code,
                        diagnostic.message
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

    /// Shows the native client's Cargo build as a separate startup phase.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn native_build_started(&mut self) -> io::Result<()> {
        self.dispatch(
            ["  Building native client with Cargo".into()],
            |dashboard| {
                dashboard.native_build_started();
            },
        )
    }

    /// Routes one Cargo progress frame or application output line to the dashboard.
    ///
    /// `message` is one carriage-return or newline-delimited child output frame.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn application_output(&mut self, message: String) -> io::Result<()> {
        let line = message.trim();
        if let Some((completed, total, target)) = cargo_progress(line) {
            return self.dispatch(
                [format!("  Cargo {completed}/{total}: {target}")],
                |dashboard| {
                    dashboard.native_build_progress(target, Some((completed, total)));
                },
            );
        }
        if let Some(target) = line.strip_prefix("Compiling ") {
            let target = target.split_whitespace().next().unwrap_or(target);
            return self.dispatch([format!("  Compiling {target}")], |dashboard| {
                dashboard.native_build_progress(target, None);
            });
        }
        if line.starts_with("Running ") {
            return self.dispatch([format!("  {line}")], |dashboard| {
                dashboard.native_build_finished();
            });
        }
        self.note(message, Color::Gray)
    }

    /// Redraws the elapsed native build time once per second while Cargo is active.
    ///
    /// # Errors
    ///
    /// Returns terminal output errors.
    pub fn native_build_tick(&mut self) -> io::Result<()> {
        if self.dashboard.native_build_tick() {
            self.draw()?;
        }
        Ok(())
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

    /// Drains pending input, redraws on resize, and returns whether Ctrl+C requested exit.
    ///
    /// # Errors
    ///
    /// Returns a terminal event or redraw failure.
    pub fn pump_input(&mut self) -> io::Result<bool> {
        if self.terminal.is_none() {
            return Ok(false);
        }
        let mut resized = false;
        while event::poll(std::time::Duration::ZERO)? {
            match event::read()? {
                Event::Resize(_, _) => resized = true,
                Event::Key(key)
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    return Ok(true);
                }
                _ => {}
            }
        }
        if resized {
            self.draw()?;
        }
        Ok(false)
    }
}

/// Extracts Cargo's exact completed/total count and current target from one progress frame.
///
/// `line` is a trimmed Cargo output frame; returns `None` for other output.
fn cargo_progress(line: &str) -> Option<(usize, usize, &str)> {
    let rest = line.strip_prefix("Building [")?.split_once("] ")?.1;
    let (count, target) = rest.split_once(':')?;
    let (completed, total) = count.trim().split_once('/')?;
    let completed = completed.trim().parse().ok()?;
    let total = total.trim().parse().ok()?;
    Some((completed, total, target.trim()))
}

impl Drop for DevConsole {
    fn drop(&mut self) {
        if let Some(terminal) = &mut self.terminal {
            let _ = terminal.show_cursor();
            let _ = io::stderr().execute(LeaveAlternateScreen);
        }
        if self.owns_raw_mode {
            let _ = disable_raw_mode();
        }
    }
}
