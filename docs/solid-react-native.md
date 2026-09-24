# Solid and React native gallery

The gallery in `apps/gallery` renders Argui primitives through one Rust host.
Solid Universal and React reconcile into the same transaction protocol; only
`argui-host` materializes native `Element` descriptions, and `argui-runtime`
owns the canonical `UiTree` on the UI thread. JavaScript does not own layout,
paint, text shaping, or the accessibility tree.

Applications written directly in Rust still use the existing `Render`/`AppModel`
path and need no JavaScript engine. QuickJS is required only when the application
uses the Solid or React TSX adapters; both paths share the native renderer.

Every built-in TSX primitive accepts the same typed accessibility properties.
Custom components can compose roles, labels, states, relations, focus and
actions without a framework-specific accessibility tree. See the
[accessibility guide](ui/accessibility.md).

## Build and run

Build the QuickJS application:

```sh
./scripts/build-gallery.sh
```

The command builds the Solid and React bundles, schema contract, and native
QuickJS executable. Bun runs build scripts and TypeScript tests only; it is
not shipped as an application runtime. Use `./scripts/linux-hidden-display.sh`
for native Linux checks and inspect a saved capture. The application uses
neither a browser nor a WebView.

## Runtime decision

QuickJS is the sole JavaScript runtime for the native gallery and the current
mobile path. This supersedes the Bun/`deno_core` runtime comparison in the
committed [architecture plan](SOLID_REACT_ARCHITECTURE_PLAN.md). Both desktop
prototypes ran the same Solid bundle, but neither produced a viable mobile
package here: Bun's standalone targets do not include Android or iOS, and the
pinned `rusty_v8` Android ARM64 archive needed by `deno_core` returned HTTP
404. No Deno Android APK or iOS proof exists. Maintaining those hosts would
add code without validating the cross-platform target, so their runner code
and Gradle profile were removed. This is a project selection based on this
prototype, not a claim that V8 is inherently unavailable on mobile.

The gallery starts with Button, Select, Animation Lab, and a Media page. Its
topbar changes light/dark mode and blue/violet/emerald accent. The sidebar
navigates without discarding global page state. Animation Lab covers the
implicit, timeline, composition, spring, and held-keyframe scenes from the
former widget gallery. Continuous motion is declared through typed native loop
properties. Rust advances the timelines on display frames; pause and resume
preserve their phase without a JavaScript timer or a host transaction per frame.
User actions retarget the discrete examples, and the engine interpolates those
changes. Leaving the page unmounts its native motions.

## Media imports

Place an SVG, PNG, JPEG, or WebP source below `apps/gallery/assets`, then run
`bun apps/gallery/scripts/generate-assets.mjs`. The generated TypeScript map
exports typed `AssetRef` values for `<svg source={...} />` and
`<image source={...} />`. The same manifest embeds the selected bytes in the
QuickJS runner. Rust decodes the files and registers renderer handles
before the first frame. The included Tabler icons are plain SVG assets, so the
native vector renderer can tint them; no framework-specific DOM icon component
is involved. An image ID is restricted to JavaScript's exact integer range,
and an unknown ID is rejected before a host commit.

Video playback is not exposed as an Argui primitive in this tranche. A real
player needs decoded frames, presentation timestamps, an audio clock, bounded
queues, GPU texture updates, lifecycle handling, and mobile packaging. Argui's
existing image/SVG asset registry does not provide those operations. GStreamer
offers [decode/playback components](https://gstreamer.freedesktop.org/documentation/playback/index.html)
and [an application sample sink](https://gstreamer.freedesktop.org/documentation/app/appsink.html),
with [platform packages](https://gstreamer.freedesktop.org/documentation/installing/)
for the desktop and mobile targets; adopting it requires a dedicated native
media integration and device validation. There is no nonfunctional `Video`
component or video-file import in this release.

## Platform validation boundary

The desktop Linux gallery uses the native Argui renderer. The Android
`quickjs` profile launches the same Solid TSX gallery through `NativeActivity`
with embedded fonts and assets. On a Pixel 8a (Android API 37, 1080 × 2400),
the app filled the available viewport, navigated between pages, opened Select,
changed its value by touch, and rendered SVG and raster media. Button activation
updated its counter; disabled and busy buttons did not activate. Animation Lab
scrolled to its spring and held-keyframe scenes. These checks used saved
captures, direct touch input, and an Android log without a fatal app error.
The default Android profile still runs the Rust widget gallery.

The [Bun executable targets](https://bun.sh/docs/bundler/executables) omit
Android and iOS. The pinned `rusty_v8` archive required for this Android ARM64
build returned HTTP 404; there is no Deno mobile RAM result. QuickJS is the
only installed JavaScript gallery runtime on the device.

Android's `uiautomator` exposed the gallery as one `android.view.View`; it did
not expose individual Button, Select, or navigation nodes. The installed
AccessKit Winit Android adapter expects `GameActivity$InputEnabledSurfaceView`,
and TalkBack was not enabled for this test. Accessibility semantics and keyboard
behavior are covered by host and desktop tests, but Android screen-reader
behavior remains unverified. The iOS shell still packages the Rust widget
gallery; this JavaScript gallery has not been launched under UIKit or tested
with VoiceOver.

After the full-viewport correction, `dumpsys meminfo` reported the following
QuickJS gallery samples on the Pixel. PSS and RSS include native rendering,
graphics, Java activity overhead, and the embedded runtime; they are not
QuickJS heap measurements. Graphics dominates the totals and needs a separate
surface/swapchain profile before treating this as an acceptable mobile budget.

| Screen | Total PSS | Total RSS | Native heap PSS | Graphics PSS |
| --- | ---: | ---: | ---: | ---: |
| Button | 474,826 KiB | 572,140 KiB | 83,510 KiB | 326,136 KiB |
| Animation Lab | 447,893 KiB | 546,324 KiB | 86,002 KiB | 292,704 KiB |

Earlier samples with a partially filled viewport are not directly comparable
because the rendered surface size changed.

## Why Animation Lab can still stutter

The current touch-scroll path is native: Android delivers pointer movement to
`argui-runtime`, which updates the Rust scroll offset and schedules a frame.
The renderer then prepares and draws the retained scene. QuickJS does not
receive a scroll transaction for each movement. The Button loading rotation
and the continuous Animation Lab scenes also advance in Rust from declarative
loop properties; they do not call Solid or React once per animation frame.
QuickJS still participates when a component mounts, a control changes state,
or navigation changes the page. Those events can affect switching latency,
but the available trace does not implicate JavaScript execution in continuous
scroll or loop-frame stutter.

On the Pixel 8a debug build, an active Animation Lab scroll still damaged the
full 1080 × 2400 surface (about 2.59 million pixels). In diagnostic samples,
frame preparation took about 5.7–6.5 ms, GPU command encoding 20.7–28.2 ms,
the render phase 27–40 ms, and the UI phase 5.3–6.2 ms. SurfaceFlinger frame
intervals had a 21.48 ms p95 and 32.48 ms p99 in that scroll run. These are
different timing scopes and must not be added together. They show a real
native render/frame-pacing problem for this scene; they do not measure a
QuickJS bottleneck. The user can still see stutter in the loading indicator
and Animation Lab, and Animation Lab scrolling is still not consistently
smooth. This work is unfinished.

Paint culling reduced work for offscreen children: in one 161-node fixture,
fewer than 20 subtrees needed painting after a scroll. On the device, visible
GPU layers fell from 37 to 14 in a comparable view. With loops paused, scroll
encoding fell from roughly 19–21 ms to 6.7–7.7 ms, and the full paused frame
from roughly 36–41 ms to 12–15 ms. Active loops still force enough repaint
to miss the frame budget. An experimental scroll-copy damage path encoded
around 31 ms in its measured case, so it was removed rather than retained as
an unproven optimization. The previously installed experimental APK also
showed transient layout corruption during scrolling. After removing that
path, the user confirmed that scrolling no longer breaks the layout. This
does not resolve the remaining frame stutter or prove that the experimental
path was the only cause of the visual defect.

The earlier Rust Widget Gallery and the new QuickJS gallery share the Rust
renderer, but they are different scenes and were measured with different
surface sizes, feature sets, and visual effects. The old gallery appearing
smoother is plausible; the available comparison cannot attribute that
difference to QuickJS. To test that claim, render the *same* Animation Lab
scene with a direct Rust `AppModel` and with the TSX host on the same device,
build profile, viewport, and animation state. Record JS execution/host
transactions, UI time, prepare/encode/present time, damage area, and
SurfaceFlinger frame intervals for both runs. If no per-frame host transaction
appears and both runs miss frames in native rendering, the fix belongs in the
Rust renderer or frame scheduling. A Rust `AppModel` path remains available
without QuickJS.

Accessibility updates also needed separation from the frame loop. The
retained semantic tree now skips rebuilding when only decorative compositor
motion changes. A direct diagnostic measured about 195–263 microseconds for
the guard over 59 semantic nodes, compared with roughly 3.35–3.7 ms for an
unconditional rebuild in the same idle scene. Focus, semantic changes, and
moving accessible controls still require updates. This improvement does not
by itself establish Android TalkBack support.

## Measured native tree cost

On an Intel Core Ultra 5 125H (Linux x86_64, Rust 1.98.0), five release-mode
test process runs mounted 10,000 text children, changed one text property, and
moved the last child before the first. Combined host commit plus `UiTree::update`
times were 4.2–12.3 ms for the isolated property (median 8.7 ms) and
7.0–18.3 ms for the move (median 9.3 ms). The host rematerialized two nodes
(the root and changed child). The tree recognized 9,999 shared subtrees for
the property change and classified the move at the parent in one visit. Reusing
the tree index allocation and ID slices reduced typical move cost, but one
sample still crossed a 16.7 ms frame budget. These are diagnostic process
samples with visible machine-load variance, not latency percentiles or
input-to-presentation measurements; large reorderable lists still need a
dedicated frame profile.

## Desktop JavaScript engine samples behind the decision

Before selecting QuickJS, the same Solid gallery bundle was run on Linux x86_64
with three native hosts. The Bun and Deno runners are no longer in the build.
The binaries below are **debug, unstripped** artifacts; their size is not a
release package size. RSS was sampled five seconds after mounting the Button
page. Bun runs as an application process plus a Bun child, so its memory is
the sum of both processes.

| Host | Debug binary | RSS at 5 s | Mount timing available |
| --- | ---: | ---: | --- |
| Bun | 506,523,456 B | 154,432 kB, both processes | Process start to first native batch: median 137.4 ms, range 90.1–151.9 ms over five runs. |
| `deno_core` | 662,526,920 B | 150,564 kB | Solid mount in an initialized V8 isolate: 42 ms in one debug test. |
| QuickJS | 511,766,336 B | 134,648 KiB | Full test including mount, interactions, and disposal: 138–150 ms; this only bounds mount from above. |

These timing scopes differ. They cannot select the fastest cold-start host yet;
a release build and the same start-to-first-frame probe are needed. The RSS
figures are diagnostic samples, not peak memory or heap-only measurements. The
QuickJS row was resampled after the responsive gallery changes on a private
1600 × 1200 Mutter display, five seconds after the Button page mounted. The
Bun and Deno rows remain older prototype samples and are not directly
comparable to this build.

A separate same-display run started the Rust Widget Gallery and QuickJS gallery
in sequence, sampling `/proc/<pid>/smaps_rollup` at five seconds and CPU time
for the following ten seconds. Both are development builds; the Rust gallery
uses all Cargo features, while the QuickJS gallery has its own dependency set.
They show process cost for two different Button pages, not an isolated JS heap
or a matched widget workload.

| App | RSS | PSS | Private clean + dirty | CPU, fraction of one core |
| --- | ---: | ---: | ---: | ---: |
| Rust Widget Gallery, all features | 173,820 KiB | 113,954 KiB | 97,360 KiB | 0.2% |
| QuickJS Solid gallery, Button | 134,648 KiB | 104,614 KiB | 94,056 KiB | 1.2% |

The QuickJS Button page still used a JavaScript timer for its loading icon in
this historical sample. The current loading icon uses a native rotation loop;
its device CPU and frame cost are measured separately.

The benchmark is reproducible with
`cargo nextest run --release -p argui-host --all-features --test transaction --offline --nocapture`.
The gallery path also has tests for the bundled Solid app, React parity and
real embedded-engine mounts; a device input-to-presentation measurement is
still needed for the runtime decision.
