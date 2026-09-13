mod app;
mod launch;
mod navigation;
mod numeric_expression;
mod pages;
mod property_slider;

pub(crate) use app::WidgetGallery;

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch::launch()?;
    Ok(())
}

#[cfg(target_os = "android")]
fn main() {
    let _launch_from_native_activity = launch::launch_android;
}
