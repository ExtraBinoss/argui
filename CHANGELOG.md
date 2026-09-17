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
- Added retained GPU canvases for editor, visualization, game/map and scientific
  viewports. `Element::gpu_canvas`, `GpuCanvasSpec` and the renderer-neutral
  `CustomPaintContext::gpu_canvas` helper preserve normal Argui layout,
  transforms, clipping, rounded corners, opacity, effects, overlays,
  interaction, accessibility, multi-window surfaces and native popups.
- Added `GpuCanvasRegistration`, `GpuCanvasRegistry`, `GpuCanvasFactory`,
  `GpuCanvasRenderer` and private-field device/render contexts, plus the exact
  `argui::render::wgpu` re-export. Callbacks can create resources from Argui's
  selected device, perform queue writes, and encode compute/render/copy work
  into a borrowed offscreen target. Argui remains the owner of backend
  selection, surface acquisition, command submission and presentation.
- Added explicit content-revision and resize/DPI caching with a configurable,
  bounded per-surface texture budget. Paused or unchanged canvases avoid custom
  GPU callbacks. Required/optional WGPU features and direction-aware limits are
  negotiated before device creation, including every Windows fallback attempt;
  incompatible shared devices fail clearly. Recoverable frame errors use a
  visible placeholder and deduplicated failure/recovery runtime events instead
  of aborting surrounding UI. Profiles, inspector traces and DevTools report
  canvas cache, render, hit, failure, byte and CPU-encode statistics.
- Added native and WebAssembly/WebGPU support plus the product-shaped
  **GPU Canvas Lab** in `app_examples/gpu-canvas`. It demonstrates an Argui
  toolbar and inspector around custom WGPU compute and render passes, pan/zoom,
  pause/resume, keyboard alternatives, overlays, an effect layer, resize/HiDPI,
  bounded particles and in-app simulated failure recovery.
- Published a release-built WebAssembly version of the GPU Canvas Lab on the
  website's App Examples page, with build/copy validation and a responsive
  compact layout for narrow viewports.
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
- Added an interaction API decision guide with a callback selection table and
  an exact-source live example that distinguishes `on_change`, `on_commit`, and
  `on_click`.
- Added a Widget Gallery drag-and-drop board with live data reordering, retained
  image cards, pointer capture, keyboard/accessibility alternatives, and
  velocity-driven squash, stretch, shadow, and spring settling.
- Added a working Widget Gallery split-pane showcase with horizontal, vertical,
  trailing, and nested IDE layouts, all resizable by pointer or keyboard.
- Added a Technicalities performance guide explaining `Immediate` versus
  `FrameCoalesced`, its requestAnimationFrame-shaped scheduling contract, and
  appropriate high-frequency workloads.

### Fixed

- Replaced unsupported checklist glyphs in the clean-code documentation example
  with explicit status labels, and made each completion step visible in the
  button label before its final disabled state.
- Rebuilt the custom-element lesson as a recognizable video-editor timeline
  with labeled tracks, fitted clip text, draggable clips, time ruler, timecode,
  and a forgiving draggable, keyboard-accessible playhead.
- Made mouse text selection start from the nearest selectable text when a drag
  begins in non-interactive whitespace, matching browser-style document
  selection without stealing button, editor, or custom gesture input.
- Embedded Arabic, Hebrew, and emoji fallback fonts in every shipped WebAssembly
  application so localized text no longer renders as missing-glyph squares.
- Applied coalesced split-pane gesture deltas to the latest controlled value so
  pane sizes no longer drift when the view rerenders during a drag.
- Coalesced drag-and-drop updates to one delivery per rendered frame, skipped
  redundant list mutations, and replaced the expensive offscreen blur with
  transform-locked shadow plates so fast drags remain responsive and artifact-free.
- Fixed frame-coalesced pan delivery to accumulate every raw delta between
  display frames, keeping split-pane handles exactly under the pointer during
  fast input on Firefox and other high-rate hosts.
- Smoothed the Widget Gallery drag-and-drop velocity deformation with
  frame-rate-independent interpolation while keeping its position and live
  velocity label on the same frame-coalesced callback.
- Clamped, wrapped, and clipped split-pane panel text and raised compact vertical
  pane minima so content cannot paint outside its panel.
- Made Android and iOS render edge to edge while preserving native safe areas,
  painting the active theme behind transparent system regions and matching
  Android system-icon contrast to light and dark themes.
- Prevented a closed or animated DevTools dock from reserving the bottom system
  inset, which removed the moving band of repeated framebuffer pixels on
  Android without hiding content behind the status or navigation bars.
- Made mobile search reliably focus and open the software keyboard, preserved
  adjustable selection handles, and stopped a touch scroll from activating the
  item released beneath the finger.
- Cleared workspace crate artifacts and the temporary crates.io registry before
  archive checks, preventing same-version caches from masking coordinated
  workspace changes and breaking dependent archive verification.
- Used an operating-system lock for coverage runs so a cached lock file from a
  cancelled CI job cannot block the next quality check.
- Limited individual Nextest cases to two minutes so a stalled native test is
  reported by name instead of consuming the full CI job timeout.
- Stopped hidden or minimized GTK windows from keeping the event loop in a
  permanent redraw poll, and queued synthetic lifecycle input through its GDK
  window instead of re-entering the renderer from a GTK signal callback.
- Kept embedded GPU canvases renderable beneath the website loading overlay so
  Chromium can initialize WebGPU and emit its ready signal instead of stalling
  a hidden iframe.
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
- Made vertical canvas dragging follow the content by default, added an
  in-app natural/inverted direction toggle, aligned keyboard panning with the
  selected direction and smoothed wheel/pinch zoom interaction.
- Advanced strict DevTools GPU-trace JSON to `argui-gpu-trace-v4` so exported
  frames include GPU-canvas cache, byte, render, hit, failure and CPU-encode
  metrics; older strict trace versions remain rejected on import.
- Made the base release gallery the recommended local command. Optional native
  integrations can still be enabled individually or together when needed.

### Known limitations

- Native file picking is not wired to Android's document provider or the iOS
  document picker yet. The cross-platform file-picker widget still compiles on
  mobile and reports the mode as unsupported; adding the two native adapters
  does not require a change to its public model.
- GPU canvases use Argui-owned WGPU instances, devices, queues and surfaces;
  the high-level runtime still does not accept externally owned GPU objects.
- Canvas pixels have no automatic semantic meaning. Applications must provide
  labels, keyboard controls and semantic Argui overlays for important actions.
- Canvas profiling measures cache behavior and CPU encoding time, not the
  duration of application-authored GPU passes; applications may encode their
  own supported timestamp queries when needed.

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
