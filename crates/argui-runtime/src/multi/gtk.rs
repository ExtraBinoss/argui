use super::MultiApplication;
use crate::{
    RuntimeError,
    app::Application,
    event::UserEvent,
    host::{HostId, LoopControl, WindowFactory},
};
use std::cell::Cell;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget},
    platform::run_return::EventLoopExtRunReturn,
};

struct GtkLoop<'a> {
    target: &'a EventLoopWindowTarget<UserEvent>,
    exit: Cell<bool>,
}

impl LoopControl for GtkLoop<'_> {
    fn exit(&self) {
        self.exit.set(true);
    }
}

impl WindowFactory for GtkLoop<'_> {
    fn open(&self, runtime: &mut Application) {
        runtime.initialize_gtk(self.target, self);
    }
}

pub(crate) fn launch(mut application: MultiApplication) -> Result<(), RuntimeError> {
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    crate::hot_reload::connect(proxy.clone().into());
    application.set_event_proxy(proxy);
    event_loop.run_return(|event, target, control| {
        *control = ControlFlow::Wait;
        let context = GtkLoop {
            target,
            exit: Cell::new(false),
        };
        match event {
            Event::NewEvents(cause) => {
                if cause == StartCause::Init {
                    for spec in application.config.windows.clone() {
                        application.open_window(&context, spec);
                    }
                    application.sync_tray();
                    application.process_pending(&context);
                }
                for entry in application.windows.values_mut() {
                    entry.runtime.wake_due_animation();
                }
            }
            Event::WindowEvent {
                window_id, event, ..
            } => {
                if let Some(key) = application.by_native.get(&HostId::Gtk(window_id)).cloned() {
                    let close = matches!(event, WindowEvent::CloseRequested);
                    if let Some(entry) = application.windows.get_mut(&key) {
                        entry.runtime.gtk_event(event, &context);
                    }
                    application.process_pending(&context);
                    if close {
                        application.handle_close(&key, &context);
                    }
                }
            }
            Event::RedrawRequested(window_id) => {
                if let Some(key) = application.by_native.get(&HostId::Gtk(window_id)).cloned() {
                    if let Some(entry) = application.windows.get_mut(&key) {
                        entry.runtime.begin_gtk_redraw();
                        entry.runtime.gtk_geometry();
                        entry.runtime.redraw(&context);
                    }
                    application.process_pending(&context);
                }
            }
            Event::MainEventsCleared => {
                for entry in application.windows.values_mut() {
                    entry.runtime.gtk_geometry();
                    entry.runtime.gtk_pointer_boundary(&context);
                    if entry
                        .runtime
                        .native_deadline()
                        .is_some_and(|deadline| deadline <= web_time::Instant::now())
                    {
                        entry.runtime.sync_native_views();
                    }
                }
                application.process_pending(&context);
            }
            Event::UserEvent(UserEvent::Preferences {
                window,
                preferences,
            }) => {
                if let Some(entry) = application.windows.get_mut(&window) {
                    entry.runtime.gtk_preferences(preferences);
                }
            }
            #[cfg(feature = "tasks")]
            Event::UserEvent(UserEvent::TasksReady) => {
                application.tasks_ready(&context);
            }
            Event::UserEvent(UserEvent::ModelsReady) => {
                application.models_ready(&context);
            }
            #[cfg(all(feature = "hot-reload", debug_assertions))]
            Event::UserEvent(UserEvent::HotReload { generation }) => {
                application.hot_reload(generation);
            }
            Event::UserEvent(UserEvent::NativeInput { window }) => {
                if let Some(entry) = application.windows.get_mut(&window) {
                    entry.runtime.gtk_pointer_boundary(&context);
                    if let Some(window) = entry.runtime.window() {
                        window.request_redraw();
                    }
                }
            }
            #[cfg(feature = "tray")]
            Event::UserEvent(UserEvent::Tray(event)) => application.tray_event(&context, event),
            _ => (),
        }
        for entry in application.windows.values_mut() {
            if let Some(error) = entry.runtime.fatal_error.take() {
                application.fatal_error = Some(error);
                context.exit();
                break;
            }
        }
        if context.exit.get() {
            *control = ControlFlow::Exit;
        } else if application
            .windows
            .values()
            .any(|entry| entry.runtime.gtk_redraw_pending())
        {
            // Tao's Linux redraw channel does not wake a blocked GTK main
            // iteration. Poll only while a frame is queued so animations and
            // scroll physics continue, then return to event-driven waiting.
            *control = ControlFlow::Poll;
        } else if let Some(deadline) = application
            .windows
            .values()
            .filter_map(|entry| {
                [
                    entry.runtime.next_animation_deadline(),
                    entry.runtime.native_deadline(),
                ]
                .into_iter()
                .flatten()
                .min()
            })
            .min()
        {
            *control = ControlFlow::WaitUntil(deadline);
        }
    });
    application.shutdown();
    application.fatal_error.take().map_or(Ok(()), Err)
}
