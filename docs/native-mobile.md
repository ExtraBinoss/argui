# Native Android and iOS

Argui's Rust engine can be embedded in native applications. The repository's
maintained mobile sample is the Solid TSX gallery on Android: its Gradle shell
starts a Rust QuickJS host, which runs the gallery bundle through the shared
host contract. Optional `argui-runtime` entry modules support applications
that build their own native integrations. This repository does
not include an iOS app shell or Xcode project.

## Support status

CI currently proves that:

- the `argui-runtime/android` entry and gallery QuickJS host cross-compile for Android;
- Gradle creates a debug APK and unsigned release AAB for the Android gallery;
- the `argui-runtime/ios` entry cross-compiles for iOS.

CI does not package an iOS application. Physical-device startup, input,
lifecycle, TalkBack/VoiceOver, document services, and owner signing still need
device validation. Treat the mobile integrations as opt-in preview support
until those checks are complete.

## Rust application integration

For a Rust application, keep its model and view in a normal Rust library. The
library can expose the construction function used by its desktop or mobile
launchers:

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

A native Rust launcher calls `argui_runtime::run_application_with_text_engine`;
mobile launchers call the equivalent function through runtime's mobile modules.
Embed resources needed at startup with `include_bytes!`.

Mobile entry features are explicit and target-gated. A Rust application selects
the engine and platform features it uses:

```toml
[dependencies]
argui-core = "0.4.0"
argui-runtime = "0.4.0"

[target.'cfg(target_os = "android")'.dependencies]
argui-runtime = { version = "0.4.0", features = ["android"] }

[target.'cfg(target_os = "ios")'.dependencies]
argui-runtime = { version = "0.4.0", features = ["ios"] }
```

The TSX application path is separate: `apps/gallery` shares the Rust host and
renderer while its Solid or React bundle defines the application UI. See the
[gallery guide](../apps/gallery/README.md).

## Android

`argui-runtime::mobile::android` connects Android's `NativeActivity` to the Winit lifecycle.
A Rust application crate builds a `cdylib` named `main` and exports the activity
entry symbol:

```toml
[lib]
name = "main"
crate-type = ["cdylib"]
```

```rust,ignore
use argui_runtime::mobile::android::AndroidApp;

fn launch(android_app: AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    let (app, renderer, text, model) = shared_app::build();
    argui_runtime::mobile::android::run_application_with_text_engine(
        android_app,
        app,
        renderer,
        text,
        model,
        |_| {},
    )?;
    Ok(())
}

argui_runtime::android_main!(launch);
```

The repository shell is in `mobile/android`. Its small Java `NativeActivity`
subclass sets up the edge-to-edge gallery; Winit and Argui render the UI. The
Gradle build compiles `apps/gallery/quickjs-host` for the selected ABIs and
packages it as `libmain.so`. Bun builds the Solid or React TSX bundle; it is not
embedded in the app.

```sh
./scripts/android-gallery.sh apk
./scripts/android-gallery.sh install
./scripts/android-gallery.sh launch
./scripts/android-gallery.sh aab
```

Install the Android SDK, NDK, JDK 17, Rust target, and `cargo-ndk` first. See
[the Android shell guide](../mobile/android/README.md) for versions, artifact
paths, emulator ABIs, and signing. After installing a debug APK once, TSX edits
can use [native hot reload](../apps/gallery/README.md#native-tsx-hot-reload)
without rebuilding Rust or reinstalling the app.

The runtime creates the window when Android resumes. Suspension drops the WGPU
surface before the native window and recreates both on resume. Text fields use
Winit's native IME path; focusing a field requests Android's soft keyboard, and
losing window focus hides it. A custom iOS host using Winit uses its UIKit
first-responder path when the app loses and regains focus.

The Android shell also packages a `dataSync` foreground service for explicitly
started background work. `argui_platform::mobile::MobileActivity` starts it
with an ongoing progress notification and a monochrome icon. On Android 13 and
later, the app asks for notification permission before starting; if the user
declines, enable notifications in system settings and try again. Start this API
from a visible user action. Android 15 and later limit background `dataSync`
foreground-service use to a cumulative six hours per 24-hour period; the
notification does not make arbitrary or indefinite background work permitted.
Other Android shells do not need the repository helper to run Argui, but must
package `crates/argui-runtime/src/mobile/android/java` and declare the service,
permissions, and notification icon before calling `MobileActivity::begin`.

## iOS

`argui-runtime::mobile::ios` exports a C-callable Rust entry point. A consuming application
builds a static library:

```toml
[lib]
crate-type = ["staticlib"]
```

```rust,ignore
fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let (app, renderer, text, model) = shared_app::build();
    argui_runtime::mobile::ios::run_application_with_text_engine(
        app,
        renderer,
        text,
        model,
        |_| {},
    )?;
    Ok(())
}

argui_runtime::ios_main!(start_argui_app, launch);
```

The consuming Xcode project calls `start_argui_app()`; Winit starts
`UIApplicationMain` and owns the UIKit window. The application owner provides
the Xcode project, bundle identity, icons, deployment target, signing, and
provisioning. There is no iOS sample shell or packaging script in this
repository.

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
the current scale. Older Android versions use the legacy window-inset API. A
Winit iOS host compares the window with its safe-area bounds. Insets refresh
with geometry and scale. The repository Android shell uses transparent system
bars, so an application can paint edge to edge. Pad interactive content while
continuing the themed background behind system bars, the home indicator, and
cutouts. Overlays that are visually closed must not reserve an inset-sized flex
item; an open overlay should paint its own safe-area padding rather than
exposing the renderer clear color. `WindowConfig::with_safe_area_insets` and
`AppCommand::SetSafeAreaInsets` support custom hosts and previews. See
[safe areas](platform/window-insets.md) for the coordinate contract.

## Direct-touch scrolling

Touch scrolling follows the finger and keeps momentum by default. This affects
direct touch only; mouse wheels and trackpads retain their platform direction.
The runtime keeps a contact pending until it exceeds
`PointerSettings::touch_slop`: small finger jitter therefore remains a tap.
Once scrolling starts, Argui locks the contact to the topmost scroll viewport
matching its dominant axis. A captured drag, such as a slider or table column
separator, wins over default scrolling for that contact. Applications that
intentionally want reversed touch movement can opt out per scroll region:

```rust,ignore
let config = ScrollConfig::default().natural_touch_scroll(false);
```

## Background activity progress

The shared API offers one owner for a native progress surface and a cloneable
reporter suitable for background work:

```rust,ignore
let mut activity = argui_platform::mobile::MobileActivity::begin(
    "Download",
    "Starting",
)?;
let progress = activity.progress();
// A worker may call progress.update(percent, "Downloading…") as work proceeds.
activity.finish()?;
```

The portable contract is semantic rather than visual. Rust shares the activity
title, current message, bounded progress, completion, and lifetime. Each native
adapter renders that state according to its operating system:

| Concern | Shared Rust | Android | iOS |
| --- | --- | --- | --- |
| Task state and updates | `MobileActivity` and `MobileActivityProgress` | foreground-service adapter | ActivityKit/UIKit adapter |
| System surface | common title/message/progress | ongoing notification | Live Activity or time-limited background task |
| Background lifetime | explicit owner and finish | foreground-service rules | ActivityKit and UIKit limits |

Applications should map platform-only decoration—notification channels, small
icons, colors, accessibility descriptions, and Dynamic Island regions—in their
native adapter. If a Live Activity cannot be started, iOS falls back to UIKit's
bounded background-task time allowance; that fallback does not keep the process
alive indefinitely. Persist resumable work and treat native progress as a
status surface rather than a durable job scheduler. This repository no longer
ships a gallery page demonstrating background activity.

## Feature limits

The Rust engine, layout, text, image/vector resources, effects, tasks, WGPU,
touch, keyboard, and IME are shared across supported native targets. The TSX
gallery currently packages desktop and Android hosts; CI only cross-checks the
iOS entry module. Desktop backdrops, native popovers, tray, desktop WebView, and
updater are platform integrations. File-picker requests have no mobile
document-provider adapter, and clipboard requests report unavailable. One
full-screen Argui window is the supported mobile application model.

## Verify cross-compilation

```sh
rustup target add aarch64-linux-android aarch64-apple-ios

cargo check --locked --target aarch64-linux-android -p argui-runtime --features android
cargo ndk -t arm64-v8a -P 26 check --locked \
  --manifest-path apps/gallery/quickjs-host/Cargo.toml --lib

cargo check --locked --target aarch64-apple-ios -p argui-runtime --features ios
```

CI packages Android artifacts only. Device validation must cover startup,
touch, text entry, rotation, background/foreground, surface recovery, shutdown,
and the platform screen reader before mobile support leaves preview status.
