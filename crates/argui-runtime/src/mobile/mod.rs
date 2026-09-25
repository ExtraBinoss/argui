//! Opt-in application entry points for native mobile targets.

#[cfg(all(feature = "android", target_os = "android"))]
pub mod android;
#[cfg(all(feature = "ios", target_os = "ios"))]
pub mod ios;
