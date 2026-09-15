# Native Android and iOS

Argui uses the same models, widgets, layout, text, and WGPU renderer on desktop,
WebAssembly, Android, and iOS. Mobile needs a small native shell for the process
entry point and packaging; application UI remains Rust.

## Support status

CI proves:

- the supported feature set cross-compiles for Android and iOS;
- Gradle creates a debug APK and unsigned release AAB;
- Xcode creates a device/simulator XCFramework and unsigned Simulator app;
- the Widget Gallery uses the same shared application on both targets;
- safe-area values reach `WindowEnvironment`.

Physical-device startup, input, lifecycle, TalkBack/VoiceOver, document services,
and owner signing still need device validation. Treat mobile as opt-in preview
support until those checks are complete.

## Share the application

Keep models, views, fonts, and renderer configuration in a normal Rust library:

```rust,ignore
pub fn build() -> (
    ApplicationConfig,
    RendererConfig,
    TextEngine,
    impl AppModel,
) {
    // Construct the application used by every launcher.
}
```

Desktop calls `argui::runtime::run_application_with_text_engine`. Android and
iOS call the equivalent function from their entry crates. Embed resources needed
at startup with `include_bytes!`.

Mobile entry crates are explicit dependencies. They are not enabled by
`argui --all-features`:

```toml
[dependencies]
argui = { version = "0.2.1", features = ["widgets-all"] }
argui-android = "0.2.1" # Android application only
# argui-ios = "0.2.1"  # iOS application only
```

## Android

`argui-android` connects Android's `NativeActivity` to the Winit lifecycle.
The application crate builds a `cdylib` named `main` and exports the activity
entry symbol:

```toml
[lib]
name = "main"
crate-type = ["cdylib"]
```

```rust,ignore
use argui_android::AndroidApp;

fn launch(android_app: AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    let (app, renderer, text, model) = shared_app::build();
    argui_android::run_application_with_text_engine(
        android_app,
        app,
        renderer,
        text,
        model,
        |_| {},
    )?;
    Ok(())
}

argui_android::android_main!(launch);
```

The repository shell is in `mobile/android`. It uses `NativeActivity`, so the
UI needs no Java or Kotlin layer.

```sh
./scripts/android-gallery.sh apk      # debug-signed installable APK
./scripts/android-gallery.sh install
./scripts/android-gallery.sh launch
ARGUI_ANDROID_ABIS=arm64-v8a ./scripts/android-gallery.sh aab
```

Install the Android SDK, NDK, JDK 17, Rust target, and `cargo-ndk` first. See
[the Android shell guide](../mobile/android/README.md) for versions, artifact
paths, emulator ABIs, and signing.

The runtime creates the window when Android resumes. Suspension drops the WGPU
surface before the native window and recreates both on resume.

## iOS

`argui-ios` exports a C-callable Rust entry point. The application crate builds
a static library:

```toml
[lib]
crate-type = ["staticlib"]
```

```rust,ignore
fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let (app, renderer, text, model) = shared_app::build();
    argui_ios::run_application_with_text_engine(
        app,
        renderer,
        text,
        model,
        |_| {},
    )?;
    Ok(())
}

argui_ios::ios_main!(start_argui_app, launch);
```

The Xcode target calls `start_argui_app()`; Winit starts `UIApplicationMain`
and owns the UIKit window. Xcode still owns bundle identity, icons, deployment
target, signing, and provisioning.

```sh
./scripts/ios-widget-gallery.sh all
```

This creates the device/simulator XCFramework and an unsigned Simulator app.
A physical-device archive or TestFlight upload needs an Apple certificate and
provisioning profile. See [the iOS shell guide](../mobile/ios/README.md).

## Safe areas

`WindowEnvironment::safe_area_insets` reports logical-pixel top, right, bottom,
and left insets. Apply them to readable content while keeping a full-bleed
background outside:

```rust,ignore
fn view(environment: WindowEnvironment) -> Element {
    full_bleed_background(
        app_content().safe_area(environment.safe_area_insets),
    )
}
```

Android derives the value from the activity content rectangle; iOS compares the
window with Winit's safe-area bounds. Insets refresh with geometry and scale.
`WindowConfig::with_safe_area_insets` and
`AppCommand::SetSafeAreaInsets` support custom hosts and previews. See
[safe areas](platform/window-insets.md) for the coordinate contract.

## Feature limits

Models, layout, text, images, vectors, effects, i18n, tasks, widgets, WGPU,
touch, keyboard, and IME cross-compile on mobile.

Desktop backdrops, native popovers, tray, desktop WebView, updater, and native
hot reload are desktop integrations. The file-picker widget compiles but has no
mobile document-provider adapter; clipboard requests report unavailable. One
full-screen Argui window is the supported mobile application model.

## Verify cross-compilation

```sh
rustup target add aarch64-linux-android aarch64-apple-ios

cargo check --target aarch64-linux-android -p argui -p argui-android \
  --features argui/i18n,argui/tasks,argui/widgets-all,argui/devtools
cargo check --target aarch64-linux-android -p argui-widget-gallery --lib

cargo check --target aarch64-apple-ios -p argui -p argui-ios \
  --features argui/i18n,argui/tasks,argui/widgets-all,argui/devtools
cargo check --target aarch64-apple-ios -p argui-widget-gallery --lib
```

CI also packages Android and iOS artifacts. Device validation must cover startup,
touch, text entry, rotation, background/foreground, surface recovery, shutdown,
and the platform screen reader before mobile support leaves preview status.
