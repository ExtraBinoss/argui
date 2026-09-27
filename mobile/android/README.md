# Run the Solid gallery on Android

This project packages the Solid TSX gallery with embedded QuickJS as a native
Android app. A small `NativeActivity` subclass draws behind the system bars;
Winit and Argui render the UI. Bun builds the JavaScript bundle, and Gradle
builds the Rust shared library into the APK or AAB.

## Native surfaces

Android represents long-running, user-visible work with a foreground service
and an ongoing notification. The shared Rust API owns the activity title,
status, progress, lifetime, and worker-facing update handle. The Java adapter
under `crates/argui-runtime/src/mobile/android/java/dev/argui/android` maps that
state to Android's notification template and service lifecycle.

The gallery draws edge to edge with transparent status and navigation bars.
Argui detects Android `WindowInsets`, including display cutouts, and exposes
them to Rust window views as `WindowEnvironment::safe_area_insets`. Applications
apply that padding with `Element::safe_area`; detection alone does not move the
gallery's TSX controls away from system bars. Native callers can override the
detected logical-pixel values with `WindowConfig::with_safe_area_insets(Insets)`.
See [safe areas](../../docs/platform/window-insets.md) for the contract.

The Android shell packages a `dataSync` foreground service, but the current
gallery has no **Background activity** page or control that starts it. An
application can start the service through `MobileActivity::begin` from a visible
user action. Android 13 and later may ask for notification permission. See
[mobile background activity](../../docs/native-mobile.md#background-activity-progress)
for the Rust API and platform limits.

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
The [Solid and React host contract](../../docs/solid-react-native.md) describes
how the bundle reaches the Rust renderer.

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
