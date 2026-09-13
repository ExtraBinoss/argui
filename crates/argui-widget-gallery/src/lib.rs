#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod app;
mod launch;
mod navigation;
mod numeric_expression;
mod pages;
mod property_slider;

pub use app::WidgetGallery;
pub use launch::launch;

#[cfg(target_os = "android")]
argui_android::android_main!(launch::launch_android);

#[cfg(target_os = "ios")]
argui_ios::ios_main!(start_argui_widget_gallery, launch::launch);
