//! iOS Live Activity integration with a finite UIKit background-task fallback.

use std::{
    ffi::{CString, c_char, c_void},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
};

use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
use objc2_ui_kit::{UIApplication, UIBackgroundTaskInvalid};

use super::MobileActivityError;

/// Starts a Live Activity when the shell's Swift bridge is present, with a UIKit fallback.
///
/// # Arguments
/// * `title` — static title shown by the Live Activity.
/// * `message` — initial progress text shown by the Live Activity.
///
/// # Returns
/// An owner that ends the started native activity.
///
/// # Errors
/// Returns [`MobileActivityError::Native`] if UIKit cannot create a background task or an input
/// contains an interior NUL byte.
pub(super) fn start(title: &str, message: &str) -> Result<IosActivity, MobileActivityError> {
    IosActivity::start(title, message)
}

/// Sends progress to a Live Activity started by the supplied opaque handle.
///
/// # Arguments
/// * `identifier` — handle returned by the ActivityKit bridge.
/// * `percent` — current progress from 0 through 100.
/// * `message` — short status text displayed by the Live Activity.
///
/// # Errors
/// Returns [`MobileActivityError::Native`] if ActivityKit rejects the update or if the message
/// contains an interior NUL byte. The function is a no-op when a shell omitted its Swift bridge.
pub(super) fn update(
    identifier: u64,
    percent: u8,
    message: &str,
) -> Result<(), MobileActivityError> {
    IosActivity::update(identifier, percent, message)
}

#[derive(Debug)]
enum ActivityTarget {
    /// ActivityKit owns a visible Live Activity through the optional Swift bridge.
    LiveActivity(u64),
    /// UIKit granted a finite background-execution assertion.
    BackgroundTask(Arc<AtomicUsize>),
    /// The activity was already finished.
    Finished,
}

/// Owns one iOS Live Activity or finite UIKit background-task assertion.
#[derive(Debug)]
pub(super) struct IosActivity {
    target: ActivityTarget,
}

impl IosActivity {
    /// Starts ActivityKit when its Swift bridge is linked, otherwise requests UIKit background time.
    ///
    /// # Arguments
    /// * `title` — static title displayed by the Live Activity.
    /// * `message` — initial status text displayed by the Live Activity.
    ///
    /// # Returns
    /// An owner that ends the Live Activity or releases its UIKit task identifier.
    ///
    /// # Errors
    /// Returns [`MobileActivityError::Native`] when UIKit cannot create a background task or
    /// when a title or message contains an interior NUL byte.
    #[allow(unsafe_code)]
    pub(super) fn start(title: &str, message: &str) -> Result<Self, MobileActivityError> {
        let title = CString::new(title).map_err(|error| native_error(error.to_string()))?;
        let message = CString::new(message).map_err(|error| native_error(error.to_string()))?;

        if let Some(bridge) = activity_bridge() {
            // SAFETY: Swift exports this C ABI and accepts borrowed, NUL-terminated strings for
            // the duration of the call. The bridge copies both strings before returning.
            let identifier = unsafe { (bridge.start)(title.as_ptr(), message.as_ptr()) };
            if identifier != 0 {
                return Ok(Self {
                    target: ActivityTarget::LiveActivity(identifier),
                });
            }
        }

        let task = begin_background_task()?;
        Ok(Self {
            target: ActivityTarget::BackgroundTask(task),
        })
    }

    /// Returns the ActivityKit identifier used by a cloneable progress reporter, if available.
    ///
    /// # Returns
    /// `Some` for an ActivityKit Live Activity, or `None` for a UIKit fallback or finished task.
    pub(super) const fn progress_id(&self) -> Option<u64> {
        match &self.target {
            ActivityTarget::LiveActivity(identifier) => Some(*identifier),
            ActivityTarget::BackgroundTask(_) | ActivityTarget::Finished => None,
        }
    }

    /// Updates this ActivityKit Live Activity with its latest progress text and percentage.
    ///
    /// # Arguments
    /// * `identifier` — handle returned when the Live Activity started.
    /// * `percent` — progress from 0 through 100; values above 100 are clamped.
    /// * `message` — short status text displayed by the widget extension.
    ///
    /// # Errors
    /// Returns [`MobileActivityError::Native`] if ActivityKit rejects the update or if the
    /// message contains an interior NUL byte. A missing bridge is treated as a no-op so an app
    /// shell without the optional widget extension remains usable.
    #[allow(unsafe_code)]
    pub(super) fn update(
        identifier: u64,
        percent: u8,
        message: &str,
    ) -> Result<(), MobileActivityError> {
        let Some(bridge) = activity_bridge() else {
            return Ok(());
        };
        let message = CString::new(message).map_err(|error| native_error(error.to_string()))?;
        // SAFETY: The bridge accepts a scalar handle, percentage, and borrowed C string, and
        // copies the string before returning.
        let result = unsafe { (bridge.update)(identifier, percent.min(100), message.as_ptr()) };
        if result == 0 {
            return Err(native_error("ActivityKit rejected the progress update"));
        }
        Ok(())
    }

    /// Ends the owned Live Activity or releases the finite UIKit background-task assertion.
    ///
    /// # Errors
    /// Returns [`MobileActivityError::Native`] if the native bridge cannot finish an ActivityKit
    /// activity or UIKit cannot release its task identifier.
    #[allow(unsafe_code)]
    pub(super) fn finish(&mut self) -> Result<(), MobileActivityError> {
        let target = std::mem::replace(&mut self.target, ActivityTarget::Finished);
        match target {
            ActivityTarget::LiveActivity(identifier) => {
                let Some(bridge) = activity_bridge() else {
                    return Err(native_error("ActivityKit bridge is no longer available"));
                };
                // SAFETY: The scalar identifier was issued by the paired Swift bridge.
                let result = unsafe { (bridge.finish)(identifier) };
                if result == 0 {
                    return Err(native_error("ActivityKit rejected the finish request"));
                }
            }
            ActivityTarget::BackgroundTask(task) => end_background_task(task)?,
            ActivityTarget::Finished => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ActivityBridge {
    start: unsafe extern "C" fn(*const c_char, *const c_char) -> u64,
    update: unsafe extern "C" fn(u64, u8, *const c_char) -> i32,
    finish: unsafe extern "C" fn(u64) -> i32,
}

/// Looks up the Swift bridge without making a shell that omits it fail to link.
#[allow(unsafe_code)]
fn activity_bridge() -> Option<ActivityBridge> {
    let start = dynamic_symbol(b"argui_ios_activity_start\0")?;
    let update = dynamic_symbol(b"argui_ios_activity_update\0")?;
    let finish = dynamic_symbol(b"argui_ios_activity_finish\0")?;
    // SAFETY: Swift exports these exact C signatures. Symbols are called only after all functions
    // are found in the current process image.
    Some(unsafe {
        ActivityBridge {
            start: std::mem::transmute::<
                *mut c_void,
                unsafe extern "C" fn(*const c_char, *const c_char) -> u64,
            >(start),
            update: std::mem::transmute::<
                *mut c_void,
                unsafe extern "C" fn(u64, u8, *const c_char) -> i32,
            >(update),
            finish: std::mem::transmute::<*mut c_void, unsafe extern "C" fn(u64) -> i32>(finish),
        }
    })
}

/// Resolves a symbol from the main executable or its linked frameworks.
#[allow(unsafe_code)]
fn dynamic_symbol(name: &'static [u8]) -> Option<*mut c_void> {
    // `RTLD_DEFAULT` is the Darwin dlsym pseudo-handle defined by `<dlfcn.h>` as `(void *)-2`.
    const RTLD_DEFAULT: *mut c_void = (-2_isize) as *mut c_void;
    // SAFETY: `name` is a static NUL-terminated symbol name and RTLD_DEFAULT is the documented
    // Darwin pseudo-handle used to search the current process and its dependencies.
    let symbol = unsafe { dlsym(RTLD_DEFAULT, name.as_ptr().cast()) };
    (!symbol.is_null()).then_some(symbol)
}

/// Requests UIKit's finite background execution time and returns its shared active-task token.
fn begin_background_task() -> Result<Arc<AtomicUsize>, MobileActivityError> {
    on_main(begin_background_task_on_main)
}

/// Starts a UIKit assertion and ends it automatically if the system's finite time expires.
fn begin_background_task_on_main(
    main_thread: MainThreadMarker,
) -> Result<Arc<AtomicUsize>, MobileActivityError> {
    let application = UIApplication::sharedApplication(main_thread);
    let task_identifier = Arc::new(AtomicUsize::new(invalid_task_identifier()));
    let expiration_identifier = Arc::clone(&task_identifier);
    let expiration_handler = RcBlock::new(move || {
        let identifier = expiration_identifier.swap(invalid_task_identifier(), Ordering::AcqRel);
        if identifier != invalid_task_identifier() {
            end_expired_task(identifier);
        }
    });
    let identifier =
        application.beginBackgroundTaskWithExpirationHandler(Some(&expiration_handler));
    if identifier == invalid_task_identifier() {
        return Err(native_error(
            "UIApplication could not begin a background task",
        ));
    }
    task_identifier.store(identifier, Ordering::Release);
    Ok(task_identifier)
}

/// Ends an active UIKit assertion from its owner or expiration callback.
fn end_background_task(task_identifier: Arc<AtomicUsize>) -> Result<(), MobileActivityError> {
    let identifier = task_identifier.swap(invalid_task_identifier(), Ordering::AcqRel);
    if identifier == invalid_task_identifier() {
        return Ok(());
    }
    on_main(move |main_thread| {
        UIApplication::sharedApplication(main_thread).endBackgroundTask(identifier);
        Ok(())
    })
}

/// Ends an expiring UIKit assertion immediately or queues its release on the main thread.
fn end_expired_task(identifier: usize) {
    if let Some(main_thread) = MainThreadMarker::new() {
        UIApplication::sharedApplication(main_thread).endBackgroundTask(identifier);
    } else {
        DispatchQueue::main().exec_async(move || {
            if let Some(main_thread) = MainThreadMarker::new() {
                UIApplication::sharedApplication(main_thread).endBackgroundTask(identifier);
            }
        });
    }
}

/// Runs UIKit work synchronously on the main thread and returns its result.
fn on_main<T: Send + 'static>(
    operation: impl FnOnce(MainThreadMarker) -> Result<T, MobileActivityError> + Send + 'static,
) -> Result<T, MobileActivityError> {
    if let Some(main_thread) = MainThreadMarker::new() {
        return operation(main_thread);
    }

    let (sender, receiver) = mpsc::sync_channel(1);
    DispatchQueue::main().exec_sync(move || {
        let result = MainThreadMarker::new().map_or_else(
            || Err(native_error("UIKit work did not run on the main thread")),
            operation,
        );
        let _ = sender.send(result);
    });
    receiver
        .recv()
        .map_err(|_| native_error("the UIKit main-thread operation was interrupted"))?
}

/// Returns UIKit's platform-defined invalid background-task identifier.
#[allow(unsafe_code)]
fn invalid_task_identifier() -> usize {
    // SAFETY: UIKit exports UIBackgroundTaskInvalid as a process-wide constant of NSUInteger type.
    unsafe { UIBackgroundTaskInvalid }
}

/// Converts a message into a native mobile-activity diagnostic.
fn native_error(message: impl Into<String>) -> MobileActivityError {
    MobileActivityError::Native(message.into())
}

#[link(name = "System")]
#[allow(unsafe_code)]
unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
