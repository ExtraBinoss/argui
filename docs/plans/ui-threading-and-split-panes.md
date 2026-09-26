# UI threading and future split panes

Status: architecture proposal. A future video editor is the motivating use
case; this document does not start an editor implementation.

## Recommendation

Keep Argui's general animation sampling and layout on the UI event-loop thread
for now. A split pane needs a layout result that agrees immediately with pointer
hit testing, focus, accessibility, and the image being presented. Moving every
layout pass to a worker would add snapshot copies, synchronization, and stale
result handling to that interaction. Moving every animation to a worker would
add cross-thread frame messages even for transforms that already use retained
composition.

This is a decision to measure a specific split-pane workload before changing
thread ownership, not a claim that UI-thread layout will always be fast enough.

## What Argui already does

- [The frame router](../../crates/argui-runtime/src/app/frame.rs) separates
  layout, paint, scroll, and composition and records their CPU time.
- [The animation scheduler](../../crates/argui-runtime/src/animation.rs)
  samples one monotonic clock on the UI thread and sleeps when idle.
- [Retained composition](../rendering/compositor.md) reuses layout and painted
  content when only a transform or group opacity changes. The renderer receives
  resolved values; it does not own animation timelines.
- [Owned native tasks](../runtime/tasks.md) can run independent work elsewhere
  and return bounded, cancellable results to the UI thread. Browser tasks do
  not yet create Web Workers.
- The UI event layer has a hidden
  [split-size mapping](../../crates/argui-ui/src/event/listener/value.rs), but
  the widget package does not yet expose a split-pane component.

## Split-pane implication

Dragging a divider changes the available width or height of both panes. Text
wrapping, scroll viewports, and child geometry may therefore genuinely change:
this is layout work, not just a compositor transform. The divider's hover,
focus, and press feedback can use composition or paint without relayout.

A future split-pane component should keep one stable native tree, enforce pane
minimum sizes, capture the pointer during drag, and offer keyboard adjustment
and accessible separator semantics. Deliver drag changes at most once per
display frame using the existing
[frame-coalesced gesture path](../performance/optimizations.md), while preserving
the final pointer position on release. Give each pane a bounded viewport and
use layout boundaries only where the parent truly owns its size. Virtualize
large child lists before considering worker layout.

## Evaluation plan before changing threads

1. Build a small split-pane benchmark scene, not an editor: one pane with a
   virtualized list and one with text and a bounded GPU canvas. Include empty
   panes as a baseline. Resize the window and drag the divider at both 60 Hz
   and, when available, 120 Hz.
2. Record p50/p95/p99 input-to-present latency, frame interval, and the
   runtime's model, tree, layout, paint, surface, and GPU times. Also record
   layout count per drag frame, allocations, memory, and idle frame requests.
   Compare runs on the same machine, build profile, viewport, and content;
   follow the [performance measurement rules](../performance/optimizations.md).
3. First try the existing paths: stable keys, bounded pane content, subtree
   invalidation, virtualization, frame-coalesced drag, and compositor-only
   divider decoration. Confirm that presented divider position and hit testing
   agree after rapid reversal and release.
4. Consider a worker-layout experiment only if layout still dominates the
   missed-frame tail after step 3. Scope it to one isolated, immutable pane
   geometry snapshot. Tag requests and results with tree, viewport, font, and
   divider revisions; apply only the newest matching result. Preserve a
   synchronous path for input-critical geometry and verify focus, hit testing,
   scrolling, accessibility, cancellation, and resize.
5. Keep the worker version only if it improves p95/p99 input-to-present latency
   on the benchmark without growing memory, queues, or idle work. Otherwise
   retain UI-thread layout and document the measured limit.

## Separate future work

If a video editor is built later, media decode, seek, thumbnail and waveform
generation, and export are good worker candidates because they do not need to
own the UI tree. The existing [native viewport plan](media-and-native-viewport.md)
describes that boundary. A dedicated render-owner thread would require its own
WGPU device, surface, and window-lifetime design; it is a separate experiment
from worker layout.
