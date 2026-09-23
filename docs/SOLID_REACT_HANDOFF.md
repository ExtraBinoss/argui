# Solid/React native gallery handoff

For the current performance status and the concrete 120 Hz scroll plan, see
[`ANIMATION_LAB_120HZ_HANDOFF.md`](ANIMATION_LAB_120HZ_HANDOFF.md). This file
also preserves the earlier handoff measurements for context.

## State at handoff

Worktree: `/home/albi/.codex/worktrees/22aa/argui` on
`codex/dsl-gallery-live`. The changes are not committed. Follow `AGENTS.md`:
run its final quality gate once after implementation is complete and before
committing. The gate has not yet run in this handoff.

The `.argui` language and its implementation were removed. Solid Universal
and React use the shared Rust host and `UiTree`; QuickJS is the only application
JavaScript runtime. A direct Rust `AppModel` application still needs no JS.
The gallery includes Button, Select, Animation Lab, navigation, themes, SVG
icons, and raster assets. Native video playback is not integrated; see
`docs/rendering/native-video-slice.md` for the proposed next slice.

Animation Lab loops and the Button spinner now advance in Rust, without a JS
transaction per frame. Paint culling and a retained accessibility guard were
added. The experimental scroll-copy path was removed. The user subsequently
confirmed that scrolling no longer breaks the layout, but still sees stutter
in the spinner and Animation Lab and an unsmooth Animation Lab scroll.

## Evidence and limits at handoff

- Pixel 8a, debug build, active Animation Lab scroll: full 1080 × 2400 damage;
  preparation about 5.7–6.5 ms, command encoding 20.7–28.2 ms, render phase
  27–40 ms; SurfaceFlinger p95 21.48 ms and p99 32.48 ms in the sampled run.
  These stages have different scopes and should not be added together.
- Paint culling reduced visible GPU layers from 37 to 14 in a device profile.
  It helped a paused scene but did not meet the active-scroll frame budget.
- The accessibility guard measured 0.195–0.263 ms for 59 semantic nodes,
  versus about 3.35–3.7 ms for the former unconditional rebuild.
- Targeted `--all-features` tests reported passing: layout paint 23/23,
  runtime route/accessibility 5/5, render offscreen 1/1, schema easing 1/1.
- No controlled same-scene comparison against direct Rust `AppModel` exists.
  The old Rust gallery and new TSX gallery use the same renderer but differ
  in scene, viewport, and effects. QuickJS is not known to cause the
  continuous scroll or animation stutter; the sampled frame cost is native.
- Android TalkBack and iOS TSX behavior remain unverified. RAM figures in
  `docs/solid-react-native.md` are whole debug-process samples, not JS heap.

## Continuation findings

The comparison runner now presents the same Animation Lab tree through either
the live Solid/QuickJS host or a frozen direct Rust `AppModel` snapshot. On the
Pixel 8a, one matched sample had idle presentation medians of about 15.3 ms
for both variants and scroll medians of 18.15 ms (QuickJS) and 18.62 ms
(Rust). The live variant emitted no JavaScript batches after startup during
the sampled native animation. These observations locate the continuing frame
cost in the native scene and renderer; they do not establish a universal frame
rate. The Rust snapshot has no JavaScript event callbacks, so normal Android
installs must use the interactive QuickJS variant.

The concrete regressions found during the continuation were separate:

- The Host scanned schema properties and events during a page remount. Caching
  valid lookups reduced a measured Animation Lab-to-Button callback from about
  106 to 43 ms and first render from about 95 to 55 ms on the Pixel.
- The Host reused a Rectangle even when a descendant Text changed. Select's
  internal value advanced while its visible label stayed stale. Descendant
  identity is now checked before reuse, with a regression test and a private
  desktop capture showing the chosen value in both places.
- The Media page's 964 x 643 SVG was rasterized three times while the vector
  atlas grew from 256 to 1024. Atlas capacity is now checked before rasterizing.
  The decorative illustration is also generated as a PNG from its SVG source
  at build time; Tabler icons remain native SVGs. Measured first render after
  the Media commit fell from about 886 to 108 ms across the tested builds.
- Continuous Size and Layout gap loops caused repeated layout work. Their
  visual loops now use native transforms; clicking Change target still uses
  real width and gap transitions. Both Solid and React have the same scene
  order, including Travel before Opacity, with navigation outside the scroll
  viewport and page-keyed scroll reset.
- The under-budget offscreen texture pool evicted idle textures every 60
  frames, invalidating retained layers and causing periodic full repaints.
  It now retains them under budget, with a 61-frame regression test. Redundant
  clear passes and Draw-batch clones were removed without changing pixels.

The remaining active-scroll cost is native. A later Pixel top-of-page
Animation Lab gesture measured about 25 ms median and 42 ms p95; with Pause
loops confirmed, the same gesture was still about 25 ms median. Warm lower
content measured about 10 ms mean renderer time, while active top-page scroll
often encoded a full frame in 15-30 ms. Desktop and mobile use the same Solid
page content with responsive shell layout. The renderer still needs a faster
scroll redraw path to sustain 60/120 Hz for this scene. Do not describe the
remaining stutter as solved until it is verified visually and in frame times.
