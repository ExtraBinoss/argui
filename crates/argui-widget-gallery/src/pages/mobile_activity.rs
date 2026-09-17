use std::time::Duration;

use argui::{
    platform::mobile::{
        MobileActivity, MobileActivityCapability, MobileActivityError, MobileActivityProgress,
    },
    runtime::{
        Context, Render,
        tasks::{self, TaskSlot},
    },
    ui::{Element, length, percent},
    widgets::{Button, Progress, shadcn},
};

use crate::app::text;

/// Gallery example for a native background task and its platform progress surface.
#[derive(Default)]
pub(crate) struct MobileActivityDemo {
    slot: TaskSlot,
    activity: Option<MobileActivity>,
    progress: Option<f32>,
    status: String,
}

impl MobileActivityDemo {
    /// Starts a short task and its native Android service or iOS activity.
    fn start(&mut self, cx: &mut Context<Self>) {
        if self.slot.is_running() {
            return;
        }
        match MobileActivity::begin("Argui background task", "Starting work…") {
            Ok(activity) => {
                let reporter = activity.progress();
                self.activity = Some(activity);
                self.progress = None;
                self.status =
                    "Working for about 20 seconds. You can leave the app open in the background."
                        .into();
                if let Err(error) = cx.spawn_latest(
                    &mut self.slot,
                    run_background_activity(reporter),
                    |demo, result, cx| {
                        demo.progress = if matches!(&result, Ok(Ok(()))) {
                            Some(100.0)
                        } else {
                            demo.progress
                        };
                        demo.status = match result {
                            Ok(Ok(())) => "Finished successfully.".into(),
                            Ok(Err(error)) => format!("Task failed: {error}"),
                            Err(error) => format!("Task stopped: {error}"),
                        };
                        if let Err(error) = demo.finish_activity() {
                            demo.status
                                .push_str(&format!(" Could not remove native activity: {error}"));
                        }
                        cx.notify();
                    },
                ) {
                    let _ = self.finish_activity();
                    self.status = format!("Could not start the task: {error}");
                }
            }
            Err(MobileActivityError::NotificationPermissionRequired) => {
                self.status = "Android requested notification permission. Allow it, then tap Start again; if denied, enable notifications in app settings.".into();
            }
            Err(error) => self.status = format!("Could not start native activity: {error}"),
        }
        cx.notify();
    }

    /// Cancels the task and removes its native activity surface.
    fn cancel(&mut self, cx: &mut Context<Self>) {
        self.slot.cancel();
        self.status = "Cancelled.".into();
        if let Err(error) = self.finish_activity() {
            self.status
                .push_str(&format!(" Could not remove native activity: {error}"));
        }
        cx.notify();
    }

    /// Finishes and removes the owned native mobile activity, if one is active.
    fn finish_activity(&mut self) -> Result<(), MobileActivityError> {
        if let Some(mut activity) = self.activity.take() {
            activity.finish()?;
        }
        Ok(())
    }
}

/// Simulates 20 seconds of cancellable work and reports progress once per second.
async fn run_background_activity(progress: MobileActivityProgress) -> Result<(), String> {
    for tick in 1..=20_u8 {
        tasks::sleep(Duration::from_secs(1)).await;
        let percent = tick * 5;
        progress
            .update(percent, &format!("Working · {percent}%"))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

impl Render for MobileActivityDemo {
    /// Renders task controls, local progress, and the current native-platform behavior.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let capability = match MobileActivity::capability() {
            MobileActivityCapability::OngoingNotification => {
                "Android uses a foreground service with an ongoing progress notification."
            }
            MobileActivityCapability::LiveActivityOrTimeLimited => {
                "iOS shows a Live Activity where allowed; otherwise execution is best-effort and time-limited, without an ongoing notification."
            }
            MobileActivityCapability::ForegroundOnly => {
                "This platform does not provide a native background activity."
            }
        };
        Element::column([
            text(
                "Start, then use Home or lock the device to check the native progress surface.",
                14.0,
                theme.foreground,
                500,
            ),
            text(capability, 13.0, theme.muted_foreground, 400),
            Element::row([
                Button::new("mobile-activity-start", "Start", theme.button())
                    .enabled(!self.slot.is_running())
                    .on_click(cx.event_handler(|demo, _, cx| demo.start(cx)))
                    .build(),
                Button::new("mobile-activity-cancel", "Cancel", theme.outline_button())
                    .enabled(self.slot.is_running())
                    .on_click(cx.event_handler(|demo, _, cx| demo.cancel(cx)))
                    .build(),
            ])
            .gap(8.0),
            Progress::new(
                "mobile-activity-progress",
                "Background task progress",
                self.progress,
            )
            .build(&theme)
            .width(percent(1.0)),
            text(
                if self.status.is_empty() {
                    "Ready. The demo runs for about 20 seconds.".into()
                } else {
                    self.status.clone()
                },
                13.0,
                theme.foreground,
                400,
            ),
            Element::container([]).height(length(1.0)),
        ])
        .gap(14.0)
        .width(percent(1.0))
    }
}
