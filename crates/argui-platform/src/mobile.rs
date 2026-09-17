//! Native mobile background-activity and system-inset support.

use std::sync::{Arc, Mutex};

use crate::Insets;
#[cfg(target_os = "android")]
use argui_core::ColorScheme;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "ios")]
mod ios;

/// Physical-pixel distances from each edge of a mobile window.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalInsets {
    /// Distance from the top edge in physical pixels.
    pub top: u32,
    /// Distance from the right edge in physical pixels.
    pub right: u32,
    /// Distance from the bottom edge in physical pixels.
    pub bottom: u32,
    /// Distance from the left edge in physical pixels.
    pub left: u32,
}

impl PhysicalInsets {
    /// Converts physical edge distances to Argui logical pixels.
    ///
    /// # Arguments
    /// * `scale_factor` — physical pixels per logical pixel; invalid values fall back to `1.0`.
    ///
    /// # Returns
    /// Insets in logical-pixel top, right, bottom, left order.
    #[must_use]
    pub fn to_logical(self, scale_factor: f32) -> Insets {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        Insets::new(
            self.top as f32 / scale_factor,
            self.right as f32 / scale_factor,
            self.bottom as f32 / scale_factor,
            self.left as f32 / scale_factor,
        )
    }
}

/// The strongest background-activity behavior supported by the current platform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MobileActivityCapability {
    /// Android can display an ongoing foreground-service notification.
    OngoingNotification,
    /// iOS may show a Live Activity or use its bounded background-task fallback.
    LiveActivityOrTimeLimited,
    /// The current target has no mobile background-activity integration.
    ForegroundOnly,
}

/// Failure to start, update, or finish a native mobile activity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileActivityError {
    /// This platform does not expose the requested mobile activity behavior.
    Unsupported,
    /// Android is asking for notification permission before starting the foreground service.
    NotificationPermissionRequired,
    /// A native operating-system call failed with the supplied diagnostic.
    Native(String),
}

impl std::fmt::Display for MobileActivityError {
    /// Formats a short user-readable mobile activity error.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => formatter.write_str("mobile background activity is unsupported"),
            Self::NotificationPermissionRequired => {
                formatter.write_str("notification permission is required to start this activity")
            }
            Self::Native(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for MobileActivityError {}

/// Owns one native mobile background activity and finishes it when dropped.
#[derive(Debug)]
pub struct MobileActivity {
    active: Arc<Mutex<bool>>,
    #[cfg(target_os = "ios")]
    native: ios::IosActivity,
}

impl MobileActivity {
    /// Returns the background behavior available on the current target.
    #[must_use]
    pub const fn capability() -> MobileActivityCapability {
        #[cfg(target_os = "android")]
        {
            return MobileActivityCapability::OngoingNotification;
        }
        #[cfg(target_os = "ios")]
        {
            return MobileActivityCapability::LiveActivityOrTimeLimited;
        }
        #[allow(unreachable_code)]
        MobileActivityCapability::ForegroundOnly
    }

    /// Starts a native activity with a title and explanatory message.
    ///
    /// # Arguments
    /// * `title` — short title shown by Android's ongoing notification or iOS Live Activity.
    /// * `message` — initial progress text shown where the platform exposes task status.
    ///
    /// # Returns
    /// An owner that stops the native activity when explicitly finished or dropped.
    ///
    /// # Errors
    /// Returns [`MobileActivityError::Unsupported`] on non-mobile targets,
    /// [`MobileActivityError::NotificationPermissionRequired`] when Android asks
    /// the user to grant notification permission, or [`MobileActivityError::Native`]
    /// when the operating system rejects the request.
    pub fn begin(title: &str, message: &str) -> Result<Self, MobileActivityError> {
        #[cfg(target_os = "android")]
        {
            android::start(title, message)?;
            Ok(Self {
                active: Arc::new(Mutex::new(true)),
            })
        }
        #[cfg(target_os = "ios")]
        {
            let native = ios::start(title, message)?;
            Ok(Self {
                active: Arc::new(Mutex::new(true)),
                native,
            })
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = (title, message);
            Err(MobileActivityError::Unsupported)
        }
    }

    /// Creates a sendable progress reporter for work running on another thread.
    #[must_use]
    pub fn progress(&self) -> MobileActivityProgress {
        MobileActivityProgress {
            active: Arc::clone(&self.active),
            #[cfg(target_os = "ios")]
            native_id: self.native.progress_id(),
        }
    }

    /// Ends the native activity and removes its ongoing notification where available.
    ///
    /// # Errors
    /// Returns [`MobileActivityError::Native`] if the operating system rejects the stop request.
    pub fn finish(&mut self) -> Result<(), MobileActivityError> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*active {
            return Ok(());
        }
        *active = false;
        #[cfg(target_os = "android")]
        android::finish()?;
        #[cfg(target_os = "ios")]
        self.native.finish()?;
        Ok(())
    }
}

impl Drop for MobileActivity {
    /// Best-effort cleanup when the owner leaves scope.
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

/// A cloneable handle for updating native progress from background work.
#[derive(Clone, Debug)]
pub struct MobileActivityProgress {
    active: Arc<Mutex<bool>>,
    #[cfg(target_os = "ios")]
    native_id: Option<u64>,
}

impl MobileActivityProgress {
    /// Updates the platform activity with a percentage and message.
    ///
    /// # Arguments
    /// * `percent` — progress from 0 through 100; values above 100 are clamped.
    /// * `message` — short status text shown by Android's notification or iOS Live Activity.
    ///
    /// # Errors
    /// Returns [`MobileActivityError::Native`] if the native progress surface rejects the update.
    pub fn update(&self, percent: u8, message: &str) -> Result<(), MobileActivityError> {
        let active = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*active {
            return Ok(());
        }
        #[cfg(target_os = "android")]
        android::update(percent, message)?;
        #[cfg(target_os = "ios")]
        if let Some(native_id) = self.native_id {
            ios::update(native_id, percent, message)?;
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let _ = (percent, message);
        Ok(())
    }
}

/// Initializes the shared Android activity/JVM bridge used by safe areas and notifications.
///
/// # Arguments
/// * `android_app` — active NativeActivity handle supplied by `android-activity`.
///
/// # Errors
/// Returns a native diagnostic if JNI cannot retain the activity/JVM context.
#[cfg(target_os = "android")]
pub fn initialize_android(
    android_app: &android_activity::AndroidApp,
) -> Result<(), MobileActivityError> {
    android::initialize(android_app).map_err(MobileActivityError::Native)
}

/// Reads Android's current system-bar and display-cutout insets in logical pixels.
///
/// # Arguments
/// * `scale_factor` — physical pixels per logical pixel for the current window.
///
/// # Returns
/// Current insets, or `None` until Android has attached window insets to its decor view.
#[cfg(target_os = "android")]
#[must_use]
pub fn android_safe_area_insets(scale_factor: f32) -> Option<Insets> {
    Some(android::safe_area_insets()?.to_logical(scale_factor))
}

/// Updates Android's status- and navigation-bar icon contrast for the active theme.
///
/// Calls before [`initialize_android`] are ignored. Repeating the current scheme is a no-op.
///
/// # Arguments
/// * `scheme` — resolved application color scheme behind the transparent system bars.
#[cfg(target_os = "android")]
pub fn set_android_system_bar_color_scheme(scheme: ColorScheme) {
    android::set_system_bar_color_scheme(scheme);
}

/// Requests the Android soft keyboard through the retained NativeActivity handle.
///
/// This explicit request supplements Winit's `set_ime_allowed` policy; showing
/// input uses Android's non-implicit request so a focused editor reliably opens
/// the keyboard. Hiding input uses Android's soft-input hide request.
/// Calls before [`initialize_android`] are ignored.
///
/// # Arguments
/// * `visible` — whether to show the soft keyboard; `false` hides it.
#[cfg(target_os = "android")]
pub fn set_android_soft_input_visible(visible: bool) {
    android::set_soft_input_visible(visible);
}
