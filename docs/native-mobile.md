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
argui = { version = "0.3.2", features = ["widgets-all"] }
argui-android = "0.3.2" # Android application only
# argui-ios = "0.3.2"  # iOS application only
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
UI itself needs no Java or Kotlin layer. The optional background-activity
example includes a small Java foreground service; applications that do not use
that API can omit the helper sources, service declaration, and permissions.

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

Text fields use Winit's native IME path. When an Argui text field gains focus,
the runtime enables IME input and explicitly requests Android's soft keyboard;
losing window focus hides it. iOS uses Winit's UIKit first-responder path and
resigns/restores it as the app loses and regains focus.

The shell also declares a `dataSync` foreground service for explicitly started
background work. `argui::platform::mobile::MobileActivity` starts it with an
ongoing progress notification and a monochrome notification icon. On Android
13 and later, the app asks for notification permission before starting; if the
user declines, enable notifications in the app's system settings and try again.
Start this API from a visible user action. Android 15 and later limit background
`dataSync` foreground-service use to a cumulative six hours per 24-hour period;
the notification does not make arbitrary or indefinite background work
permitted. The Java helper/service is included by the repository's Gradle
source set and declared in its manifest. Other Android shells do not need that
helper to run Argui, but must package
`crates/argui-android/android/src/main/java` and declare the service,
permissions, and notification icon before calling `MobileActivity::begin`.

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

Android reads current `WindowInsets` from the native activity, including system
bars and display cutouts, then converts physical pixels to logical pixels using
the current scale. Older Android versions use the legacy window-inset API. iOS
compares the window with Winit's safe-area bounds. Insets refresh with geometry
and scale. Android's shell uses transparent system bars and iOS renders against
Winit's outer surface, so an application can paint edge to edge on both systems.
Paint the safe-area wrapper with the current theme background: padding protects
interactive content while the background continues behind status, navigation,
home-indicator, and cutout regions. Overlays that are visually closed must not
reserve an inset-sized flex item; an open overlay should paint its own safe-area
padding rather than exposing the renderer clear color.
`WindowConfig::with_safe_area_insets` and
`AppCommand::SetSafeAreaInsets` support custom hosts and previews. See
[safe areas](platform/window-insets.md) for the coordinate contract.

## Direct-touch scrolling

Touch scrolling follows the finger and keeps momentum by default. This affects
direct touch only; mouse wheels and trackpads retain their platform direction.
The runtime keeps a contact pending until it exceeds `PointerSettings::touch_slop`:
small finger jitter therefore remains a tap. Once scrolling starts, Argui locks
the contact to the topmost scroll viewport matching its dominant axis. A
widget-owned captured drag, such as a slider or table column separator, wins
over default scrolling for the lifetime of that contact.
Applications that intentionally want reversed touch movement can opt out per
scroll region:

```rust,ignore
let config = ScrollConfig::default().natural_touch_scroll(false);
```

## Background activity progress

The shared API offers one owner for a native progress surface and a cloneable
reporter suitable for background work:

```rust,ignore
let mut activity = argui::platform::mobile::MobileActivity::begin(
    "Download",
    "Starting",
)?;
let progress = activity.progress();
// A worker may call progress.update(percent, "Downloading…") as work proceeds.
activity.finish()?;
```

The portable contract is semantic rather than visual. Rust shares the activity
title, current message, bounded progress, completion, and lifetime. Each native
shell renders that state according to its operating system:

| Concern | Shared Rust | Android | iOS |
| --- | --- | --- | --- |
| Task state and updates | `MobileActivity` and `MobileActivityProgress` | consumed by JNI adapter | consumed by C/Swift bridge |
| System surface | common title/message/progress | ongoing notification | Lock Screen and Dynamic Island |
| Native layout | no platform markup | Android notification template | SwiftUI `ActivityConfiguration` |
| Background lifetime | explicit owner and finish | foreground service rules | ActivityKit plus finite UIKit fallback |

This boundary keeps application logic portable without pretending the native
surfaces are interchangeable. Add future shared fields to the Rust state and
bridge contract, then map them independently in the Android notification and
iOS SwiftUI extension. Platform-only decoration—notification channels, small
icons, SF Symbols, Dynamic Island regions, colors, and accessibility
descriptions—belongs to the corresponding native adapter.

Android uses the foreground service and ongoing notification. iOS uses an
ActivityKit Live Activity when the device and user settings allow it. If a Live
Activity cannot be started, iOS falls back to UIKit's bounded background-task
time allowance; that fallback has no ongoing notification and does not keep the
process alive indefinitely. iOS background execution can be suspended or ended
by the system, so persist resumable work and treat native progress as a status
surface rather than a durable job scheduler. The gallery's **Background
activity** example is available only on Android and iOS.

## Feature limits

Models, layout, text, images, vectors, effects, i18n, tasks, widgets, WGPU,
touch, keyboard, and IME cross-compile on mobile.

Desktop backdrops, native popovers, tray, desktop WebView and updater are
desktop integrations. The file-picker widget compiles but has no mobile
document-provider adapter; clipboard requests report unavailable. One
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
