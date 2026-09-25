#[cfg(all(feature = "android", target_os = "android"))]
#[path = "mobile/android.rs"]
mod android;

#[cfg(all(feature = "ios", target_os = "ios"))]
#[path = "mobile/ios.rs"]
mod ios;
