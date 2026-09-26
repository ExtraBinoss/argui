# Animation pipeline review

This review compares Argui's native and Web animation pipeline with the open
source WarpUI client at commit
[`5af88f49`](https://github.com/warpdotdev/warp/tree/5af88f49f84e70025f9c19e13f6b9ae64b624627).
The comparison is about mechanisms in source, not a claim that either client
has better frame times on the same workload. Graphical profiling was excluded
from this review.

| Stage | WarpUI mechanism | Argui finding and action |
| --- | --- | --- |
| Scheduling | The [presenter collects the earliest repaint deadline](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/presenter.rs#L603-L629), and the [window timer avoids a duplicate later wake](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/core/app.rs#L3605-L3644). | Argui's tree now reports the earliest one-shot deadline for held caret, keyframe, and Steps motion alongside continuous demand for smooth tracks ([tree](../../crates/argui-ui/src/tree/animation.rs)). The existing [runtime](../../crates/argui-runtime/src/animation.rs) routes that deadline through the native and Web event loops. A Steps plateau therefore leaves the frame scheduler idle until the next jump. `loopSteps` is exposed by the [native schema](../../crates/argui-schema/src/builtin/loop_motion.rs) and exercised in the [Solid](../../apps/gallery/src/solid/animation-page.tsx) and [React](../../apps/gallery/src/react/animation-page.tsx) galleries. |
| Motion | Warp animated images [wake after the remaining frame hold](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/elements/gui/image.rs#L322-L348). | Argui now computes one-shot step and held-keyframe deadlines in [timeline control](../../crates/argui-animation/src/timeline.rs), including delay, playback direction and rate, seek, pause/resume, retarget, and iteration boundaries. Springs and smooth keyframe segments remain display-linked. We also corrected redundant terminal change reports and exact spring settling in [motion control](../../crates/argui-animation/src/controller.rs) and [spring physics](../../crates/argui-animation/src/spring.rs). |
| Invalidation | Warp retains rendered view elements but [rebuilds layout and paint for a requested scene](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/presenter.rs#L333-L475). | Argui already distinguishes layout, paint, scroll, and composite frames ([frame router](../../crates/argui-runtime/src/app/frame.rs)). We fixed [interaction decoration](../../crates/argui-ui/src/tree.rs) so a resolved transform-only press does not carry a conservative paint flag into its first frame. Empty focus/position updates no longer rescan transitions. |
| Layout and state | Warp's retained presenter [rebuilds only invalidated views](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/presenter.rs#L318-L331). | Argui's retained tree already skips shared subtrees. We removed quadratic binding deduplication at mount and linear node-ID searches for layout transition indices during layout frames ([animation registry](../../crates/argui-ui/src/tree/animation.rs), [transition registry](../../crates/argui-ui/src/tree/transition.rs)). The transition registry now samples running nodes only. For pointer and focus events, [transition sync](../../crates/argui-ui/src/tree/transition/sync.rs) finds the outermost affected state scope and resolves only that subtree, including portal descendants; it falls back to full-tree resolution when event coverage is uncertain or layout, scrolling, or authored properties change. Thus unrelated styled siblings are not visited for an isolated click. |
| Text and damage | Warp keeps [text layouts used in the current or previous frame](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/text_layout.rs#L62-L105) and [prepares draw buffers per scene](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui/src/rendering/wgpu/renderer/frame.rs#L27-L69). | Argui has bounded text caches, retained paint fragments, and a compositor frame that skips text preparation. We reused shared text visuals and made the [damage comparison](../../crates/argui-render/src/damage/scene.rs) one pass; unchanged glyph vectors are now compared by shared identity. |
| GPU composition | Warp batches [instanced rectangles by layer](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui/src/rendering/wgpu/renderer/rect.rs#L199-L235) and [clears the full target](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui/src/rendering/wgpu/renderer/frame.rs#L95-L113). | Argui already batches contiguous draws and retains the root target with partial damage. We skipped unnecessary geometry cloning on opacity-only composite frames ([composite geometry](../../crates/argui-layout/src/composite.rs)). Porting Warp's full-scene path would discard existing retention. |

Two other WarpUI policies were inspected but not copied. Warp [expires GPU image
textures after ten unused frames](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/rendering/texture_cache.rs#L33-L93),
whereas Argui's image cache has an explicit byte budget and asset registration;
the lifetime policies serve different ownership models. Warp's [shimmering text
asks for a repaint about every 32 ms](https://github.com/warpdotdev/warp/blob/5af88f49f84e70025f9c19e13f6b9ae64b624627/crates/warpui_core/src/elements/gui/shimmering_text.rs#L225-L244),
which is a component-specific quality choice, not a suitable global frame cap.

The binding registry keeps a linear schedule check and sampling pass. A
`Motion` handle can be started, paused, resumed, retargeted, or finished by its
owner without changing the `UiTree`; the next query observes the shared
handle's state. Inactive tracks take an atomic active check before sampling
([track interface](../../crates/argui-animation/src/controller/track.rs)). An
active-only list would require subscriptions on every shared handle, cleanup
on tree rebuild and removal, and a wake path to the native/Web event loop for
changes made outside an application event. The existing scheduler checks the
tree after application events and frames; owners making an out-of-band change
must post an application update to wake an idle runtime, as for other UI state.
Simply indexing the handles would not supply that wake. The transition
registry has no external mutation path, so it safely uses an active-only list.
A [test](../../crates/argui-ui/tests/tree/animation.rs)
covers external start, pause, resume, retarget, and finish without a tree
rebuild.

The ignored [binding scan probe](../../crates/argui-ui/tests/tree/binding_benchmark.rs)
times 1,000 schedule queries and frame samples with one running track at the
end of 128, 512, 2,048, or 8,192 bindings. Run it with:

```sh
RUSTC_WRAPPER= CARGO_TARGET_DIR=target/dev cargo nextest run -p argui-ui --all-features -E 'test(binding_registry_scan_cost)' --run-ignored all --no-capture
```

One debug-build CPU run measured 2.8, 11.6, 56.1, and 223.9 µs per sample
respectively. At 512 bindings, that is about 0.07% of a 16.7 ms frame; even
8,192 bindings consume about 1.3%. Idle schedule checks measured 2.7, 7.9,
40.5, and 193.3 µs respectively and do not drive repeated frames. The scan
is measurable but small for this workload; subscriptions would add mutation,
allocation, and lifecycle work to every track without evidence of a frame
bottleneck. These are single-machine CPU samples, not browser/GPU timings.

The concrete rapid-click invariant is now: once hover paint is established, a
press or release that changes only the button transform reports `Composite`,
not `Paint`. The button surface, icon, and text share one compositor layer; the
display-list content and prepared text remain unchanged through rapid
press/release. Tests cover these conditions, a text editor focus during a
composited press, final transform paint for text sharpness, and idle frame
termination. The work reduction is deterministic: composite frames skip
text/GPU-buffer preparation, empty focus or pointer updates skip a full style
resolution, and animation sampling visits active transition nodes only. This
review does not establish an end-to-end Web frame-time or smoothness result
because no browser/GPU trace was collected.

The ignored CPU probe in
[`transition_benchmark.rs`](../../crates/argui-ui/tests/tree/transition_benchmark.rs)
constructs 32, 128, or 512 unrelated styled siblings, then times 100
press/release pairs on one sibling. Run it with:

```sh
RUSTC_WRAPPER= CARGO_TARGET_DIR=target/dev cargo nextest run -p argui-ui --all-features -E 'test(transition_click_cost)' --run-ignored all --no-capture
```

One debug-build run before subtree sync measured 114, 503, and 1,605 µs per
pair at 32, 128, and 512 siblings respectively. The same fixture after the
change measured 70, 61, and 60 µs, roughly a 27-fold reduction at 512. These
are CPU wall-clock samples from one machine, not statistical confidence
intervals or browser/GPU frame timings; the probe is retained for reproduction.
