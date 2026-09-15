use crate::{Button, Dialog, DialogAction, DialogBehavior, Progress, WidgetTheme};
use argui_text::TextStyle;
use argui_ui::{Element, LiveRegion, Role, Semantics, UiEvent, UiEventKind};
use argui_updater::{InstallOutcome, State};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// User intent emitted by the controlled update dialog.
pub enum UpdateAction {
    Open,
    Close,
    Check,
    Download,
    Cancel,
    Install,
}

/// Controlled update UI. The owner runs the engine and supplies its current state.
/// Closing the dialog hides it; only `Cancel` cancels an active download.
pub struct UpdateDialog<'a> {
    key: String,
    state: &'a State,
    open: bool,
    trigger: Element,
}

impl<'a> UpdateDialog<'a> {
    /// Creates an update dialog view backed by the updater's current `state`.
    ///
    /// `key` identifies the dialog, `open` controls its visibility, and `trigger`
    /// is the element that opens it. The owner handles returned actions.
    pub fn new(key: impl Into<String>, state: &'a State, open: bool, trigger: Element) -> Self {
        Self {
            key: key.into(),
            state,
            open,
            trigger,
        }
    }

    #[must_use]
    /// Returns the update action requested by `event`, or `None` if it is unrelated.
    pub fn action(&self, event: &UiEvent) -> Option<UpdateAction> {
        let dialog = DialogBehavior::new(&self.key, "Application update", self.open);
        if let Some(action) = dialog.action(event) {
            return match action {
                DialogAction::Open => Some(UpdateAction::Open),
                DialogAction::Close if self.open => Some(UpdateAction::Close),
                _ => None,
            };
        }
        if self.open
            && matches!(event.kind, UiEventKind::Click(_))
            && event.target_key() == Some(format!("{}::primary", self.key).as_str())
        {
            return self.primary().map(|(_, action)| action);
        }
        None
    }

    fn primary(&self) -> Option<(&'static str, UpdateAction)> {
        match self.state {
            State::Idle | State::UpToDate => Some(("Check for updates", UpdateAction::Check)),
            State::Failed(_) => Some(("Try again", UpdateAction::Check)),
            State::Available(_) | State::Cancelled(_) => {
                Some(("Download update", UpdateAction::Download))
            }
            State::Downloading { .. } | State::Verifying(_) => {
                Some(("Cancel download", UpdateAction::Cancel))
            }
            State::Ready(_) => Some(("Install update", UpdateAction::Install)),
            _ => None,
        }
    }

    #[must_use]
    /// Builds the dialog contents and controls using `theme` for styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let title = self.state.release().map_or_else(
            || "Application update".into(),
            |info| format!("Update {}", info.version),
        );
        let status = match self.state {
            State::Idle => "Check for a new version.".into(),
            State::Checking => "Checking for updates…".into(),
            State::UpToDate => "You're up to date.".into(),
            State::Available(_) => "A new version is available.".into(),
            State::Downloading { progress, .. } => {
                let downloaded = progress.downloaded as f64 / 1_000_000.0;
                progress.total.filter(|total| *total > 0).map_or_else(
                    || format!("{downloaded:.1} MB downloaded"),
                    |total| {
                        format!(
                            "{downloaded:.1} / {:.1} MB · {:.0}%",
                            total as f64 / 1_000_000.0,
                            progress.percent().unwrap_or(0.0)
                        )
                    },
                )
            }
            State::Verifying(_) => "Verifying the download…".into(),
            State::Ready(_) => "Download verified. Ready to install.".into(),
            State::Installing(_) => "Installing the update…".into(),
            State::Installed {
                outcome: InstallOutcome::RestartRequired,
                ..
            } => "Update installed. Restart the application to use it.".into(),
            State::Installed {
                outcome: InstallOutcome::InstallerLaunched,
                ..
            } => "Installer started. Save your work and close the application.".into(),
            State::Cancelled(_) => "Download cancelled. You can try again.".into(),
            State::Failed(error) => error.clone(),
        };
        let text = |content, size, color| {
            Element::text(content).text_style(TextStyle {
                font_size: size,
                line_height: size * 1.5,
                color,
                ..Default::default()
            })
        };
        let failed = matches!(self.state, State::Failed(_));
        let mut semantics =
            Semantics::new(if failed { Role::Alert } else { Role::Status }).label(&status);
        semantics.live = LiveRegion::Polite;
        let mut children = vec![text(title, 22.0, theme.foreground)];
        if let Some(info) = self.state.release().filter(|info| !info.notes.is_empty()) {
            children.push(text(info.notes.clone(), 14.0, theme.muted_foreground));
        }
        children.push(
            text(
                status,
                14.0,
                if failed {
                    theme.destructive
                } else {
                    theme.foreground
                },
            )
            .semantics(semantics),
        );
        if let State::Downloading { progress, .. } = self.state {
            children.push(
                Progress::new(
                    format!("{}::progress", self.key),
                    "Update download",
                    progress.percent(),
                )
                .build(theme),
            );
        }
        let mut buttons = Vec::new();
        if let Some((label, _)) = self.primary() {
            buttons
                .push(Button::new(format!("{}::primary", self.key), label, theme.button()).build());
        }
        buttons.push(
            Button::new(
                format!("{}::close", self.key),
                "Close",
                theme.outline_button(),
            )
            .build(),
        );
        children.push(
            Element::row(buttons)
                .gap(8.0)
                .flex_wrap(argui_ui::FlexWrap::Wrap),
        );
        Dialog::new(
            &self.key,
            "Application update",
            self.open,
            self.trigger,
            Element::column(children).gap(16.0),
        )
        .build(theme)
    }
}
