# Designing expressive motion in Argui

Read this with the [animation architecture](animation-architecture.md) before
building a loader, gradient action, voice surface, or animated status. Motion
must communicate a state and preserve a readable control. The gallery's
Expressive UI page demonstrates the native recipes below.

## Place the effect

Choose one visual focus per UI area. Give it a reason to exist:

| Situation | Treatment | State source |
| --- | --- | --- |
| A response under about two seconds | Keep the control stable; its press response is enough. | Native pressed state |
| A longer indeterminate wait | A compact throbber beside a specific verb, such as “Generating video”. | Application loading state |
| Voice capture | Color blooms at the bottom edge; keep a legible “Listening” label and stop action above it. | Recording state, optionally an audio level |
| Processing after speech | Gather the bloom into a narrower traveling beam; change the label to “Processing”. | Processing state |
| A highlighted action | One subtle edge beam and halo when active, with quiet contrast when idle. | Button/toggle state |
| Known completion fraction | A determinate progress track; do not imply a fake percentage with an endless loop. | Real progress value |

Do not fill every nearby card with a glowing effect. A spinner next to a status
line is useful; the same spinner repeated across a list becomes noise. Keep
meaningful text even when an effect is visually distinctive. Decorative layers
should not intercept pointer or keyboard interaction.

## Shape the light

A single rainbow gradient swept across a rectangle looks like a color strip.
Build a glow as **light behind an opaque surface**:

1. Start with a dark, stable rounded host whose border and content remain crisp.
2. Put several bounded radial lobes near its lower edge. Each lobe has a bright
   colored center, a middle stop with much less alpha, and a transparent stop
   of the *same hue*. Let some of each lobe extend below the clip. For an
   unmasked halo outside a control, center the radial gradient in its own
   rectangle and fade to zero at every boundary; otherwise its rectangular
   edge is visible as a hard stripe.
3. Move neighboring lobes with different native transform periods and small
   distances; a gentle scale change varies their apparent reach. Use an
   independent, narrower beam for the processing state.
4. For an edge treatment, paint a multistop OKLab gradient over a surface two
   pixels larger than the opaque interior. Move an oversized gradient within
   the clipped frame so the border never reveals an empty side.
5. Keep text, icons, and hit target above the decorative paint. Make the idle
   state quiet so activation reads as a change. Pause loops while hidden or
   inactive, and preserve a complete static state for reduced motion.

Use a handful of layers and ordered stops. Argui's linear and radial brushes
shade on the GPU with premultiplied alpha; animate the *layer transform or
opacity* when the color pattern itself need not change. Native `loopMs`,
`loopTranslateX/Y`, `loopScale`, `loopOpacity`, and `rotationLoopMs` use the
retained motion registry. The application changes a state at the click or data
event, not on every frame. Avoid animating width, gap, or padding for ambient
light because those ask layout to run.

## Tune the movement

- Give each motif a distinct rhythm. A spinner can rotate steadily in about
  1.2 seconds; a glow can drift over several seconds; a status line may breathe
  through a small opacity range. Do not make every layer oscillate in sync.
- Keep the moving gradient larger than its clipped viewport over the entire
  travel range. Large translation on a same-sized gradient exposes blank bands.
- Prefer a fixed layout box, controlled alpha, and a small transform range.
  Motion should not move a label away from its action or make an icon lag behind
  its button.
- For a throbber with text, give the activity a clear verb. Animate the dots or
  ring; let the text breathe only slightly. A loader with no status is ambiguous.
- Make state changes legible. Listening, processing, and done require distinct
  text and motion, not the same glow with a different label.

## Review it as a product surface

For a popup entrance, use native `PopupWindow.openingMs`, `openingScale` and
`openingTranslateY`, with a positive duration from 1 to 60000 ms. Keep the OS
window's geometry fixed. An anchored entrance pivots at its trigger and reverses
vertical translation when collision placement puts it above. Compact item-aligned
menus should use zero translation. Do not drive the entry with JS timers or an
`onMount` change that can merge into the first commit and skip its initial frame.
Retain equivalent finite timelines after completion as well as while running;
otherwise a hover or data update can replay the fade. A real close / remount must
start a fresh timeline. Reduced motion must immediately present the settled menu.
Verify both in-window and native surfaces: when collision placement exposes pixels
outside the retained source clip, normal repaint is required rather than an
incorrect compositor patch. Shadow padding is part of the native surface bounds.

Inspect the idle, entering, active, exiting, paused, and reduced-motion states.
Check at compact and wide sizes, in both light and dark themes. The color should
be concentrated enough to look intentional but never obscure the foreground.
Watch several full loop cycles: look for an exposed edge, a discontinuity at
turnaround, banding, flicker, or mismatched icon/text movement. If a capture is
blank or a control cannot be identified, the visual check failed.

Verify the implementation separately: both TSX adapters mount, the native
bridge accepts all brushes, and the update class is composition when a static
brush layer moves. Inspect frame timings on the target platform before calling
an effect “smooth”; a compositor classification alone does not prove 60 fps.

Placement and state naming were informed by the
[Libraries.dev skill](https://github.com/Jakubantalik/Libraries.dev/tree/main/skills/libraries-dev)
and its [Voice reference](https://libraries.dev/voice). Argui scenes use native
primitives; their motion, geometry, and performance must be checked in Argui.
