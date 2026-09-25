# Run the Solid gallery on Android

This project packages the Solid TSX gallery with embedded QuickJS as a native
Android app. A small `NativeActivity` subclass draws behind the system bars;
Winit and Argui render the UI. Bun builds the JavaScript bundle, and Gradle
builds the Rust shared library into the APK or AAB.

## Background activity UI

Android represents long-running, user-visible work with a foreground service
and an ongoing notification. The shared Rust API owns the activity title,
status, progress, lifetime, and worker-facing update handle. The Java adapter
under `crates/argui-runtime/src/mobile/android/java/dev/argui/android` maps that
state to Android's notification template and service lifecycle.

The gallery renders edge to edge. Its status and navigation bars are
transparent, and the current theme background is painted underneath them. The
native host pads text and controls by the detected Android `WindowInsets`,
including display cutouts. Native callers can override those values in logical
pixels with `WindowConfig::with_safe_area_insets(Insets)`; the Solid gallery uses
automatic detection.

To test the reference integration, open **Background activity** in the mobile
gallery and start the demo. Android 13 or newer asks for notification
permission on first use. Accept it, leave the app, and verify that the progress
notification remains visible while the foreground service is active.

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
`platform-tools` directory on `PATH`. Gradle uses the checked-in wrapper.

## Build and install a debug APK

Connect an Android device with USB debugging enabled, or start an emulator.
Then run:

```sh
adb devices
./scripts/android-gallery.sh apk
./scripts/android-gallery.sh install
./scripts/android-gallery.sh launch
```

The APK is written to `mobile/android/app/build/outputs/apk/debug/app-debug.apk`
and installs as `dev.argui.solidgallery.debug`. It contains `arm64-v8a` and
`x86_64` libraries by default. To build only for an ARM64 phone:

```sh
./scripts/android-gallery.sh install -ParguiAbis=arm64-v8a
# Equivalent environment-variable form:
ARGUI_ANDROID_ABIS=arm64-v8a ./scripts/android-gallery.sh apk
```

The JavaScript gallery uses QuickJS on Android. Bun remains its TSX build tool.
The runtime decision and memory measurements are in
`docs/solid-react-native.md`.

## Build a release AAB

```sh
./scripts/android-gallery.sh aab
```

Gradle writes `mobile/android/app/build/outputs/bundle/release/app-release.aab`.
It is unsigned unless the application owner supplies a signing configuration.

## What Gradle builds

Gradle invokes `cargo ndk` for the selected Android ABIs, builds
`argui-gallery-quickjs` with its `android_main` entry point, and places the
result in the ABI-specific `jniLibs` folder as `libmain.so`. Debug builds use
Cargo's development profile; the AAB uses `--release`. The shared Cargo target
directory stays at the repository root.

To remove Android build outputs without deleting the Rust workspace cache:

```sh
./scripts/android-gallery.sh clean
```
