# iOS Widget Gallery packaging

The app's primary interface is the existing Rust `argui-widget-gallery` UI:
Winit starts UIKit and Argui draws the application window. A small Swift
ActivityKit bridge and WidgetKit extension provide the system-owned Live
Activity on iOS 16.1 and later; its attributes, bridge, and extension sources
live in `crates/argui-ios/swift/activity-kit/{shared,app,extension}`. The Xcode
shell references those sources and embeds the extension in the app.

The gallery's mobile Activity example appears only on mobile. On supported
iOS versions, it starts a Live Activity showing its title, current status, and
progress in the Lock Screen and Dynamic Island. If Live Activities are
unavailable or a shell does not link the Swift bridge, `argui-platform` falls
back to `UIApplication.beginBackgroundTask`; that grants finite extra runtime
for cleanup and does not keep an iOS process alive indefinitely.

## ActivityKit UI and source ownership

ActivityKit requires the system surface to be declared with SwiftUI inside a
WidgetKit extension. Argui therefore shares activity **state**, not rendered
pixels, across Android and iOS. Rust owns the title, status message, progress,
lifetime, and worker-facing update handle. The extension turns that state into
the platform-specific Lock Screen and Dynamic Island layouts.

All Swift sources intentionally stay under one crate-owned tree:

```text
crates/argui-ios/swift/activity-kit/
├── shared/ArguiActivityAttributes.swift  # data contract used by both targets
├── app/ArguiActivityBridge.swift         # C ABI called by Rust
└── extension/ArguiActivityWidget.swift   # SwiftUI system surfaces
```

`ArguiActivityWidget.swift` supplies four presentations from the same content
state:

- a branded Lock Screen card with title, live status, progress bar, percentage,
  and completion treatment;
- expanded Dynamic Island leading, trailing, center, and bottom regions;
- compact leading/trailing progress;
- a minimal circular progress indicator.

To customize appearance, edit only the extension view. To add shared data,
first extend `ContentState` in the shared file, then populate it in the app
bridge and consume it in the extension. Keep attributes for values fixed for
the activity lifetime and content state for values that update. Both the app
and extension targets must compile the shared file; only the app target owns
the bridge, and only the extension target owns the `@main` widget.

Do not attempt to render the Argui/WGPU application tree inside a Live
Activity. iOS owns this constrained surface and expects SwiftUI. Keep business
state and progress calculation in Rust, while keeping native layout, SF
Symbols, accessibility labels, and Dynamic Island composition in SwiftUI.

The main app renders against the outer iOS surface so backgrounds extend edge
to edge. Safe-area padding keeps controls outside the status bar, Dynamic
Island/notch, and home indicator; paint that wrapper with the active theme so
the system-edge regions remain visually continuous instead of becoming black.

The checked-in extension is a working reference rather than a requirement for
every Argui app. A shell that omits it still runs the main Rust UI and uses the
finite UIKit background-task fallback, but it cannot show a Live Activity.

## Build artifacts

On macOS, install Xcode and its command-line tools, then run:

```sh
./scripts/ios-widget-gallery.sh simulator
```

The command builds three Rust static libraries (arm64 iOS device, arm64 iOS
Simulator and x86_64 iOS Simulator), combines the simulator architectures,
packages both platform variants into
`mobile/ios/build/ArguiWidgetGallery.xcframework`, and builds an unsigned
`ArguiWidgetGallery.app` for the iOS Simulator, including the ActivityKit
WidgetKit extension. The app is at:

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
architectures, including the Rust-side UIKit fallback and optional ActivityKit
bridge interface. A Linux cross-check does not compile Swift sources or create
Apple binaries; run `simulator` on macOS with Xcode to validate the extension
and build an installable Simulator app.

After launch, open **Background activity** in the gallery and start the demo.
On a compatible simulator, lock the simulated device to inspect the Lock
Screen view. Dynamic Island layouts require a simulator device that exposes
the island. Live Activities must also be enabled for the app in iOS settings.

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
