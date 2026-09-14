# Native Android and iOS

Argui uses the same retained models, widgets, layout, text and WGPU renderer on
desktop, WebAssembly and mobile. A mobile application is a native APK/AAB or iOS
bundle, but its interface is drawn by Argui. Application authors do not rebuild
their views with Jetpack Compose, SwiftUI or UIKit.

The operating system still requires a small native shell. `argui-android`
connects Android's `NativeActivity` to Winit. `argui-ios` exports a Rust function
that an Xcode target can call. Winit owns the UIKit window internally on iOS.

## Current status

- [x] `argui-android` configures Winit with the required `AndroidApp`.
- [x] `argui-ios` exports a configurable C entry point for an Xcode target.
- [x] WGPU selects a compatible Vulkan backend on Android and Metal on iOS.
- [x] The runtime creates windows in `resumed`.
- [x] Android suspension drops the WGPU surface before the native window and
  recreates both when the activity resumes.
- [x] Touch, keyboard and IME events use the existing Winit event path.
- [x] The Widget Gallery provides Android and iOS native entry points.
- [x] CI cross-compiles every supported facade capability and the Widget Gallery
  for `aarch64-linux-android` and `aarch64-apple-ios`.
- [x] Gradle packages an installable debug APK and an unsigned release AAB from
  the shared Widget Gallery; CI retains both as artifacts.
- [x] Xcode packages device and simulator Rust libraries as an XCFramework and
  builds an unsigned Simulator `.app`; CI retains both as zipped artifacts.
- [x] `WindowEnvironment::safe_area_insets` exposes logical-pixel Android/iOS
  safe areas, and the Widget Gallery applies them at its root.
- [ ] Run and capture the Widget Gallery on an Android emulator and physical
  device.
- [ ] Install and capture the Widget Gallery on an iOS simulator and signed
  physical-device build.
- [ ] Expose virtual-keyboard insets independently from the safe area.
- [ ] Add native clipboard, document picker and sharing adapters.
- [ ] Validate VoiceOver and TalkBack with real devices.
- [ ] Add owner-signed Android release and iOS archive/TestFlight jobs.

CI now proves the Rust graph, Android APK/AAB packaging and iOS
XCFramework/Simulator app build. Physical-device input, lifecycle,
accessibility and signing checks remain release requirements before either
target is called production-supported.

## Keep the application shared

Put models, views, embedded fonts and renderer configuration in a normal Rust
library. Return the values needed by the platform launcher:

```rust,ignore
pub fn build() -> (ApplicationConfig, RendererConfig, TextEngine, impl AppModel) {
    // Construct the same application used by desktop and WebAssembly.
}
```

Desktop calls `argui::runtime::run_application_with_text_engine`. Mobile calls
the matching function from `argui-android` or `argui-ios`. Only the final entry
point changes.

Assets needed before the native filesystem is mounted should use
`include_bytes!`, as the Widget Gallery does for fonts and its icon. Platform
document pickers can provide files after startup once their mobile adapters are
implemented.

## Android NativeActivity

The repository contains a complete example shell under `mobile/android`. With
the Android SDK, NDK, JDK 17 and `cargo-ndk` installed, build and install it with:

```sh
./scripts/android-gallery.sh apk
./scripts/android-gallery.sh install
./scripts/android-gallery.sh launch
```

`apk` creates an installable, debug-signed APK. `aab` creates an unsigned
release bundle for which the application owner must provide a Play upload key:

```sh
ARGUI_ANDROID_ABIS=arm64-v8a ./scripts/android-gallery.sh aab
```

See [`mobile/android/README.md`](../mobile/android/README.md) for SDK versions,
artifact paths, emulator ABIs and `adb` commands.

The final Android application crate owns the dynamic library. Its manifest adds
the platform bootstrap explicitly, keeping mobile out of `argui --all-features`:

```toml
[lib]
name = "main"
crate-type = ["cdylib"]

[dependencies]
argui = { version = "0.2.1", features = ["widgets-all"] }
argui-android = "0.2.1"
```

The library name is `main` because the Android manifest below loads
`libmain.so`. Define the launch function and generate the symbol required by
`android-activity`:

```rust,ignore
use argui_android::AndroidApp;

fn launch(android_app: AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    let (config, renderer, text, model) = shared_app::build();
    argui_android::run_application_with_text_engine(
        android_app,
        config,
        renderer,
        text,
        model,
        |_| {},
    )?;
    Ok(())
}

argui_android::android_main!(launch);
```

The macro creates the one unmangled Rust symbol required by Android's activity
glue. This is the only use of an unsafe attribute; it does not contain an unsafe
block and exists solely for the operating-system entry point.

A minimal `AndroidManifest.xml` uses the platform `NativeActivity`, so no Java
or Kotlin interface is required:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
  <uses-feature
      android:name="android.hardware.vulkan.level"
      android:required="false" />
  <application
      android:hasCode="false"
      android:label="Argui app">
    <activity
        android:name="android.app.NativeActivity"
        android:configChanges="orientation|keyboardHidden|screenSize|screenLayout|uiMode"
        android:exported="true">
      <meta-data
          android:name="android.app.lib_name"
          android:value="main" />
      <intent-filter>
        <action android:name="android.intent.action.MAIN" />
        <category android:name="android.intent.category.LAUNCHER" />
      </intent-filter>
    </activity>
  </application>
</manifest>
```

Install the Rust targets and `cargo-ndk`, then place each shared library in the
matching Gradle `jniLibs` directory:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk --locked
cargo ndk -t arm64-v8a -t x86_64 \
  -o android/app/src/main/jniLibs \
  build --release --lib
```

The Android SDK, NDK and Gradle still package, install and sign the application.
Use `arm64-v8a` for current devices and `x86_64` for a matching emulator. Keep
the activity alive while Winit runs; Android may call `android_main` again after
activity recreation.

## iOS static library

The repository's `mobile/ios` Xcode project uses only a small Objective-C
`main`; the interface and application state remain Rust. On macOS with Xcode:

```sh
./scripts/ios-widget-gallery.sh all
```

This builds a device/simulator XCFramework and an unsigned Simulator `.app`.
The Simulator bundle can be installed with `xcrun simctl`; a physical-device
IPA and TestFlight build require an Apple certificate, provisioning profile and
App Store Connect signing. See [`mobile/ios/README.md`](../mobile/ios/README.md)
for the exact artifact and launch commands.

The final iOS application crate owns a static library:

```toml
[lib]
crate-type = ["staticlib"]

[dependencies]
argui = { version = "0.2.1", features = ["widgets-all"] }
argui-ios = "0.2.1"
```

Use the normal Argui launcher and export one function for Xcode:

```rust,ignore
fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let (config, renderer, text, model) = shared_app::build();
    argui_ios::run_application_with_text_engine(
        config,
        renderer,
        text,
        model,
        |_| {},
    )?;
    Ok(())
}

argui_ios::ios_main!(start_argui_app, launch);
```

Build separate libraries for a physical device and Apple Silicon simulator:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo build --release --target aarch64-apple-ios --lib
cargo build --release --target aarch64-apple-ios-sim --lib
```

Add the appropriate `.a` output to the matching Xcode target. The Xcode
`main.m` calls the exported Rust function; Winit then starts
`UIApplicationMain` and creates its UIKit window:

```objc
#import <UIKit/UIKit.h>

extern void start_argui_app(void);

int main(void) {
    @autoreleasepool {
        start_argui_app();
    }
    return 0;
}
```

Link the system frameworks requested by Winit and WGPU, including UIKit, Metal
and QuartzCore. Xcode continues to own the bundle identifier, signing team,
deployment target, icons, launch screen and provisioning profiles.

## Mobile feature boundaries

The following Argui subsystems cross-compile today: retained models, layout,
text shaping, images, vectors, WGPU rendering, effects, i18n, tasks, all widgets,
Winit touch/keyboard/IME delivery and AccessKit adapters.

Desktop backdrop, native popovers, system tray, desktop WebView, application
updater and native hot reload are desktop integrations. Do not enable those
features for a mobile build. The file-picker widget compiles on mobile but stays
disabled until a document-provider adapter exists. Clipboard requests currently
return an unavailable result on mobile.

An app should treat one main full-screen window as the supported mobile model.
Multiple Argui windows remain a desktop capability until mobile presentation
semantics are designed and tested.

## Safe areas

`WindowEnvironment::safe_area_insets` reports the current top, right, bottom and
left distances in Argui logical pixels. Android derives them from Winit's
`NativeActivity` content rectangle; iOS derives them from the full window and
Winit's safe-area bounds. Insets are refreshed when the native geometry or
scale changes.

The app decides where to apply them, so full-bleed backgrounds can still draw
under system bars. Wrap the content that must stay readable:

```rust,ignore
fn view(environment: WindowEnvironment) -> Element {
    app_content().safe_area(environment.safe_area_insets)
}
```

For previews and embedded hosts, `WindowConfig::with_safe_area_insets` supplies
an explicit value and `AppCommand::SetSafeAreaInsets` updates or clears that
override at runtime. The complete API and coordinate contract are documented
in [window safe-area insets](platform/window-insets.md).

## Repository verification

The Rust checks used by CI can be run without an emulator or Xcode:

```sh
rustup target add aarch64-linux-android aarch64-apple-ios
cargo check --target aarch64-linux-android -p argui -p argui-android \
  --features argui/i18n,argui/tasks,argui/widgets-all,argui/devtools
cargo check --target aarch64-linux-android -p argui-widget-gallery --lib
cargo check --target aarch64-apple-ios -p argui -p argui-ios \
  --features argui/i18n,argui/tasks,argui/widgets-all,argui/devtools
cargo check --target aarch64-apple-ios -p argui-widget-gallery --lib
```

CI additionally runs `scripts/android-gallery.sh` to create the APK/AAB and
`scripts/ios-widget-gallery.sh all` on macOS. Successful workflow runs expose
the `argui-widget-gallery-android` and `argui-widget-gallery-ios` artifacts.

Before marking either roadmap item complete, add a captured smoke test covering
startup, touch, text entry, rotation, background/foreground, surface recovery
and clean shutdown. Accessibility checks must exercise TalkBack or VoiceOver,
not only the semantic tree in a headless test.
