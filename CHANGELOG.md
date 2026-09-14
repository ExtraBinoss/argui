# Changelog

All notable changes to Argui are documented here. The 0.x series may still make
breaking API changes; Cargo features keep optional integrations out of builds
that do not use them.

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
