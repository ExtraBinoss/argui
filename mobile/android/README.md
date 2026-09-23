# Run the Argui Widget Gallery on Android

This project packages either the Rust Widget Gallery or the Solid gallery driven
by embedded QuickJS as a native Android app.
Android's built-in `NativeActivity` starts the widget profile. The Solid profile
uses a small `NativeActivity` subclass to draw behind the system bars. Both
start Winit and Argui and contain no Java or Kotlin UI. A Java service provides
the optional ongoing background activity notification. The Rust shared library
is built from the workspace and included in the APK or AAB by Gradle.

## Background activity UI

Android represents long-running, user-visible work with a foreground service
and an ongoing notification. The shared Rust API owns the activity title,
status, progress, lifetime, and worker-facing update handle. The Java adapter
under `crates/argui-android/android/src/main/java/dev/argui/android` maps that
state to Android's notification template and service lifecycle.

This is deliberately separate from the iOS SwiftUI layout: Android and iOS do
not share a native view system, but both consume the same semantic state. The
Android shell must package the Java adapter and declare its service,
foreground-service permissions, notification permission, and monochrome small
icon. Applications that never call `MobileActivity::begin` may omit all of
those optional pieces.

The Solid profile renders edge to edge. Its status and navigation bars are
transparent, and the current theme background is painted underneath them. The
native host pads text and controls by the detected Android `WindowInsets`,
including display cutouts. Native callers can override those values in logical
pixels with `WindowConfig::with_safe_area_insets(Insets)`; the Solid gallery uses
automatic detection and does not currently expose a JavaScript inset control.
An overlay that reaches a system edge should paint its own safe-area padding
instead of exposing the renderer clear color. The widget profile retains its
existing Android activity and window behavior.

To test the reference integration, open **Background activity** in the mobile
gallery and start the demo. Android 13 or newer asks for notification
permission on first use. Accept it, leave the app, and verify that the progress
notification remains visible while the foreground service is active. A
foreground service is not permission for unlimited background execution; use
the Android service type and workload limits appropriate to the real task.

## Install the build tools

Install Android Studio or the Android command-line tools and a JDK 17 or newer,
then install these SDK packages and accept their licenses:

```sh
sdkmanager --licenses
sdkmanager "platform-tools" "platforms;android-36" \
  "build-tools;36.0.0" "ndk;28.2.13676358"
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk --locked
```

Set `ANDROID_HOME` (or `ANDROID_SDK_ROOT`) to the SDK directory and put its
`platform-tools` directory on `PATH`. Common SDK directories are
`$HOME/Android/Sdk` on Linux, `$HOME/Library/Android/sdk` on macOS, and
`%LOCALAPPDATA%\Android\Sdk` on Windows. Gradle uses the checked-in wrapper and
downloads its pinned Gradle and Android Gradle Plugin versions on first use.

## Build and install a debug APK

Connect an Android device with USB debugging enabled, or start an emulator.
Then run:

```sh
adb devices
./scripts/android-gallery.sh apk
./scripts/android-gallery.sh install
```

The debug APK is written to
`mobile/android/app/build/outputs/apk/debug/app-debug.apk` and uses Gradle's
debug signing key, so `installDebug` can install it directly. The default build
contains `arm64-v8a` and `x86_64` libraries, for current phones and common
emulators. To build only for an ARM64 phone, pass a Gradle property:

```sh
./scripts/android-gallery.sh apk -ParguiAbis=arm64-v8a
./scripts/android-gallery.sh install -ParguiAbis=arm64-v8a
# Equivalent environment-variable form, convenient in CI:
ARGUI_ANDROID_ABIS=arm64-v8a ./scripts/android-gallery.sh apk
```

To install the Solid gallery on an ARM64 phone, select its Gradle profile:

```sh
./scripts/android-gallery.sh install -ParguiAbis=arm64-v8a -ParguiGallery=solid
./scripts/android-gallery.sh launch -ParguiGallery=solid
```

The Solid profile builds the shared JavaScript bundle with Bun, embeds it in
the QuickJS Rust library, and installs as `dev.argui.solidgallery.debug`. The
default widget profile keeps `dev.argui.widgetgallery.debug`, so both apps can
coexist on the device. Set `ARGUI_ANDROID_GALLERY=solid` to select the profile
without passing a Gradle property.

The JavaScript gallery uses QuickJS on Android. Bun remains the build tool for
its TSX bundle. The runtime decision and measured memory are documented in
`docs/solid-react-native.md`.

Launch the installed gallery from the device, or use:

```sh
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk
adb shell am start -W -a android.intent.action.MAIN \
  -c android.intent.category.LAUNCHER \
  -n dev.argui.widgetgallery.debug/android.app.NativeActivity
adb logcat -s RustStdoutStderr
```

The `am start` launch command is also available as
`./scripts/android-gallery.sh launch`.

## Build a release AAB

```sh
./scripts/android-gallery.sh aab
```

Gradle writes `mobile/android/app/build/outputs/bundle/release/app-release.aab`.
It is an unsigned release bundle unless a signing configuration is supplied by
the application owner. Use a locally managed upload key before uploading a
bundle to Google Play. Do not commit keystores or passwords.

## What Gradle builds

The Android manifest selects the platform `NativeActivity` for widgets and an
edge-to-edge `NativeActivity` subclass for Solid; both load `main`.
Gradle invokes `cargo ndk` for the selected Android ABIs, builds the selected
gallery library with its `android_main` entry point, and places
the result in the ABI-specific `jniLibs` folder as `libmain.so`. Debug builds
use Cargo's development profile; the AAB uses `--release`. The shared Cargo
target directory stays at the repository root, so incremental Rust artifacts
are reused across Gradle invocations.

To remove Android build outputs without deleting the Rust workspace cache:

```sh
./scripts/android-gallery.sh clean
```
