# Run the Argui Widget Gallery on Android

This project packages the existing Rust Widget Gallery as a native Android app.
Android's built-in `NativeActivity` starts Winit and Argui; the app contains no
Java or Kotlin UI. The Rust shared library is built from the workspace and
included in the APK or AAB by Gradle.

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

Launch the installed gallery from the device, or use:

```sh
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk
adb shell monkey -p dev.argui.widgetgallery.debug 1
adb logcat -s RustStdoutStderr
```

The first command is also available as `./scripts/android-gallery.sh launch`.

## Build a release AAB

```sh
./scripts/android-gallery.sh aab
```

Gradle writes `mobile/android/app/build/outputs/bundle/release/app-release.aab`.
It is an unsigned release bundle unless a signing configuration is supplied by
the application owner. Use a locally managed upload key before uploading a
bundle to Google Play. Do not commit keystores or passwords.

## What Gradle builds

The Android manifest declares the platform `NativeActivity` and loads `main`.
Gradle invokes `cargo ndk` for the selected Android ABIs, builds the
`argui-widget-gallery` library with its `android_main` entry point, and places
the result in the ABI-specific `jniLibs` folder as `libmain.so`. Debug builds
use Cargo's development profile; the AAB uses `--release`. The shared Cargo
target directory stays at the repository root, so incremental Rust artifacts
are reused across Gradle invocations.

To remove Android build outputs without deleting the Rust workspace cache:

```sh
./scripts/android-gallery.sh clean
```
