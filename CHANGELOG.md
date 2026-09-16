# Changelog

All notable changes to Argui are documented here. The 0.x series may still make
breaking API changes; Cargo features keep optional integrations out of builds
that do not use them.

## [Unreleased]

### Added

- Added opaque `EventHandler`/typed `ValueHandler` bindings and local direct
  handlers across interactive widgets, including domain payloads for forms,
  selection, ranges, overlays, menus, navigation, data controls, and composites.
- Added the publishable `argui-testing` crate with real headless layout and hit
  testing, accessible queries, editing, focus, scrolling, gestures, lifecycle,
  multiple windows, diagnostic settle bounds, and deterministic task time.
- Added the small `argui::prelude`, `basic`, `desktop`, and `web` convenience
  feature profiles, while preserving every granular feature.
- Added staged archive verification and a public API/SemVer CI job for the 0.3
  release line.
- Added installable Android Widget Gallery packaging for ARM64 devices and
  x86-64 emulators, including launcher and notification icons, command-line SDK
  scripts, USB deployment, and native soft-keyboard integration.
- Added shared mobile background-activity state with an Android foreground
  service and ongoing progress notification, plus an iOS ActivityKit bridge and
  SwiftUI Lock Screen/Dynamic Island extension kept under `argui-ios`.
- Added direct-touch momentum, configurable natural scrolling, drag-safe click
  activation, and draggable text-selection handles shared by Android and iOS.
- Added inherited text-selection highlight styling with solid or gradient fills,
  per-corner radii and an interactive Widget Gallery page.
- Added default-on Windows renderer fallback from DirectX 12 DirectComposition
  to an opaque DirectX 12 surface and then Vulkan, with an opt-out configuration
  and actionable runtime diagnostics for every failed attempt.
- Added `default_theme` as the clear public name for Argui's standard widget
  palette while keeping `shadcn` as a compatible alias.
- Added an interactive documentation site whose lessons display and run their
  exact Rust source as dedicated WebAssembly examples, including a complete
  light/dark, accent and token override configurator, plus side-by-side solid
  and backdrop-blurred popover surfaces in the overlays lesson.
- Added dedicated Technicalities and Platforms documentation categories that
  explain retained versus immediate UI, Argui's scope, the audited target
  support matrix, and a clearly labelled Android/iOS capability roadmap.

### Fixed

- Made Android and iOS render edge to edge while preserving native safe areas,
  painting the active theme behind transparent system regions and matching
  Android system-icon contrast to light and dark themes.
- Prevented a closed or animated DevTools dock from reserving the bottom system
  inset, which removed the moving band of repeated framebuffer pixels on
  Android without hiding content behind the status or navigation bars.
- Made mobile search reliably focus and open the software keyboard, preserved
  adjustable selection handles, and stopped a touch scroll from activating the
  item released beneath the finger.
- Prevented Web canvases from taking focus and moving an embedding page while
  they load; full-page apps can opt in through `WindowConfig::focus_on_launch`.
- Kept Winit's AppKit content view attached when enabling the macOS desktop
  backdrop, so the Metal surface remains visible with `--all-features`.
- Applied GTK's integer buffer scale to the Wayland WebView canvas, preventing
  oversized and clipped rendering on HiDPI Linux desktops.
- Kept the GTK WebView event loop active while a redraw is queued, so animated
  text, scroll momentum and other continuous frames do not stall between input
  events on Linux.
- Removed the unused native WebView deadline warning on macOS and Windows.

### Changed

- Moved the workspace to 0.3.0. `Context::callback` now provides the short
  invalidating path; `event_handler`, `listener`, `Element::on`, and typed
  behavior/action APIs remain the explicit advanced layer.
- Migrated every naturally local documentation example, its exact generated
  website snippet, the Widget Gallery, and the fake AI harness to direct
  callbacks while retaining the Events delegation example and complex reducers.
- Made the base release gallery the recommended local command. Optional native
  integrations can still be enabled individually or together when needed.

### Known limitations

- Native file picking is not wired to Android's document provider or the iOS
  document picker yet. The cross-platform file-picker widget still compiles on
  mobile and reports the mode as unsupported; adding the two native adapters
  does not require a change to its public model.

## [0.2.1] - 2026-09-14

### Fixed

- Kept virtual lists responsive while models stream frequent updates, preserved
  scroll ownership during rebuilds and improved fast wheel scrolling with
  momentum and continuous animation frames.
- Corrected text antialiasing for sRGB output so light and dark themes keep a
  consistent perceived font weight.
- Preserved browser double-click word selections instead of collapsing them on
  the following frame.
- Propagated layout and paint effects requested during animation callbacks.
- Preserved each element's corner radii across hover and active overrides.
- Stabilized native popup verification when the private Linux compositor exits
  after the application has already completed its interaction assertions.

### Added

- Added the responsive AI streaming harness with variable-height virtualization,
  live telemetry and a 1,000 token-per-second simulated workload.
- Added Motion Lab examples for springs, resizing, corner radii, colors,
  translation and rotation while every scroll region remains interactive.
- Added responsive Widget Gallery navigation for narrow and touch layouts.
- Added browser-specific WebGPU recovery guidance, mobile-safe canvas sizing,
  embedded app examples, Discord navigation and repository star counts to the
  website.
- Added native lifecycle coverage for safe areas, repeated virtualized layout
  passes and the model-free application entry point. `argui-runtime` now clears
  the repository's 85% branch coverage requirement.

### Changed

- Removed the obsolete State, Actions and Motion & Loading examples.
- Moved installation and gallery launch commands directly below the README
  preview and documented opt-in feature combinations and measured package size.

## [0.2.0] - 2026-09-14

### Added

- Added optional Fluent localization through `argui-i18n`, including locale
  negotiation, fallbacks, variables, plurals and live switching in the Widget
  Gallery.
- Added optional Subsecond hot patching for native debug applications.
- Added opt-in `argui-android` and `argui-ios` entry crates, shared safe-area
  handling and CI packaging for Android APK/AAB and iOS XCFramework/Simulator
  artifacts.
- Added Windows and macOS workspace checks, Rust build caching, crates.io archive
  validation and ordered publication of all public crates.

[0.2.1]: https://github.com/ExtraBinoss/argui/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ExtraBinoss/argui/releases/tag/v0.2.0
[Unreleased]: https://github.com/ExtraBinoss/argui/compare/v0.3.0...HEAD
