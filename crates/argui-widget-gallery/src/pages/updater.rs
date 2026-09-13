use argui::{
    animation::Frame,
    runtime::{Context, Render},
    ui::{Element, EventType},
    widgets::{Button, UpdateAction, UpdateDialog, shadcn},
};
use argui_updater::{InstallOutcome, Progress, ReleaseInfo, State};

pub(crate) struct UpdaterDemo {
    state: State,
    open: bool,
    elapsed: f64,
    unknown_size: bool,
}

impl Default for UpdaterDemo {
    fn default() -> Self {
        Self {
            state: State::Available(release()),
            open: false,
            elapsed: 0.0,
            unknown_size: false,
        }
    }
}

fn release() -> ReleaseInfo {
    ReleaseInfo {
        version: "1.2.0".into(),
        notes: "Faster startup, smoother scrolling and improved keyboard navigation.".into(),
    }
}

impl UpdaterDemo {
    fn event(&mut self, event: &argui::ui::UiEvent, cx: &mut Context<Self>) {
        let dialog = UpdateDialog::new(
            "update-demo",
            &self.state,
            self.open,
            Element::container([]),
        );
        match dialog.action(event) {
            Some(UpdateAction::Open) => self.open = true,
            Some(UpdateAction::Close) => self.open = false,
            Some(UpdateAction::Check) => {
                self.elapsed = 0.0;
                self.state = State::Checking;
            }
            Some(UpdateAction::Download) => {
                self.elapsed = 0.0;
                self.state = State::Downloading {
                    release: release(),
                    progress: Progress {
                        downloaded: 0,
                        total: (!self.unknown_size).then_some(96_000_000),
                    },
                };
            }
            Some(UpdateAction::Cancel) => self.state = State::Cancelled(release()),
            Some(UpdateAction::Install) => {
                self.elapsed = 0.0;
                self.state = State::Installing(release());
            }
            None if matches!(event.kind, argui::ui::UiEventKind::Click(_)) => match event
                .target_key()
            {
                Some("update-demo-error") => {
                    self.state = State::Failed(
                        "Unable to reach the update server. Try again when you're online.".into(),
                    );
                    self.open = true;
                }
                Some("update-demo-current") => {
                    self.state = State::UpToDate;
                    self.open = true;
                }
                Some("update-demo-unknown") => {
                    self.unknown_size = !self.unknown_size;
                }
                _ => return,
            },
            None => return,
        }
        event.stop_propagation();
        cx.notify();
    }
}

impl Render for UpdaterDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        super::preview(
            "Application updates",
            "Preview a sample update. This demo does not change your application.",
            Element::column([
                UpdateDialog::new(
                    "update-demo",
                    &self.state,
                    self.open,
                    Button::new("update-open", "Preview update", theme.button()).build(),
                )
                .build(theme),
                Element::row([
                    Button::new(
                        "update-demo-error",
                        "Connection error",
                        theme.outline_button(),
                    )
                    .build(),
                    Button::new(
                        "update-demo-current",
                        "Already up to date",
                        theme.outline_button(),
                    )
                    .build(),
                    Button::new(
                        "update-demo-unknown",
                        if self.unknown_size {
                            "Use known size"
                        } else {
                            "Use unknown size"
                        },
                        theme.outline_button(),
                    )
                    .build(),
                ])
                .gap(8.0)
                .flex_wrap(argui::ui::FlexWrap::Wrap),
            ])
            .gap(16.0),
            theme,
        )
        .on(cx.listener(EventType::Click, Self::event))
        .on(cx.listener(EventType::Key, Self::event))
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        if !self.wants_animation_frame() {
            return;
        }
        self.elapsed += frame.elapsed.as_secs_f64();
        self.state = match &self.state {
            State::Checking if self.elapsed >= 0.5 => State::Available(release()),
            State::Downloading { progress, .. } => {
                let downloaded = (self.elapsed * 8_000_000.0).min(96_000_000.0) as u64;
                if downloaded == 96_000_000 {
                    self.elapsed = 0.0;
                    State::Verifying(release())
                } else {
                    State::Downloading {
                        release: release(),
                        progress: Progress {
                            downloaded,
                            total: progress.total,
                        },
                    }
                }
            }
            State::Verifying(_) if self.elapsed >= 0.75 => State::Ready(release()),
            State::Installing(_) if self.elapsed >= 0.75 => State::Installed {
                release: release(),
                outcome: InstallOutcome::RestartRequired,
            },
            state => state.clone(),
        };
        cx.notify();
    }

    fn wants_animation_frame(&self) -> bool {
        matches!(
            self.state,
            State::Checking
                | State::Downloading { .. }
                | State::Verifying(_)
                | State::Installing(_)
        )
    }
}
