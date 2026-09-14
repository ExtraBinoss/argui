# iOS Widget Gallery packaging

This target uses the existing Rust `argui-widget-gallery` UI. The only native
source is a short Objective-C `main` that calls the C entry point exported by
`argui-ios`; Winit starts UIKit and Argui draws the application interface.
There is no SwiftUI or UIKit view implementation.

## Build artifacts

On macOS, install Xcode and its command-line tools, then run:

```sh
./scripts/ios-widget-gallery.sh simulator
```

The command builds three Rust static libraries (arm64 iOS device, arm64 iOS
Simulator and x86_64 iOS Simulator), combines the simulator architectures,
packages both platform variants into
`mobile/ios/build/ArguiWidgetGallery.xcframework`, and builds an unsigned
`ArguiWidgetGallery.app` for the iOS Simulator. The app is at:

```text
mobile/ios/build/DerivedData/Build/Products/Debug-iphonesimulator/ArguiWidgetGallery.app
```

To launch it manually on a booted simulator:

```sh
xcrun simctl list devices available
xcrun simctl boot 'iPhone 17'
xcrun simctl bootstatus booted -b
xcrun simctl install booted mobile/ios/build/DerivedData/Build/Products/Debug-iphonesimulator/ArguiWidgetGallery.app
xcrun simctl launch booted dev.argui.widgetgallery
```

Choose a simulator name shown by `simctl list devices` if `iPhone 17` is not
installed. The build script only creates artifacts; it does not boot, install,
or launch a simulator.

To create only the reusable device/simulator XCFramework, run:

```sh
./scripts/ios-widget-gallery.sh xcframework
```

The Rust target checks can run on Linux without Xcode:

```sh
./scripts/ios-widget-gallery.sh check
```

They verify the gallery's Rust entry point for device and both simulator
architectures. A Linux cross-check does not create Apple binaries or an app.

## Device builds and TestFlight

This skeleton does not create an IPA. An iOS device app must be signed with an
Apple development or distribution certificate and use a matching provisioning
profile. TestFlight additionally requires an App Store Connect distribution
workflow and a signed archive uploaded to App Store Connect. An unsigned device
archive cannot be installed on a physical iPhone or submitted to TestFlight.

The simulator `.app` is deliberately built with Xcode code signing disabled;
simulator builds do not need device provisioning. The Rust device static
library is included in the XCFramework so a later signed Xcode archive can use
the same Argui entry point.
