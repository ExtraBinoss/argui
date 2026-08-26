# Interaction model

Winit events are translated into renderer-independent platform events. The
runtime converts physical pointer coordinates to logical UI coordinates, then
`argui-ui` dispatches them against the latest clipped Taffy geometry.

```text
winit -> argui-platform -> argui-runtime -> argui-ui
                                              |
                                     hit test and state
                                              |
                              repaint display list only
```

Every retained element receives a stable `NodeId`. Unkeyed nodes retain their ID
at the same structural position; keyed siblings retain it when reordered. IDs
are never derived from hashes, and duplicate keys never produce duplicate IDs.
Application-facing `UiEvent`s carry both the ID and the optional key.

Hit regions are emitted in paint-tree order and tested in reverse, so the last
interactive element painted wins. Bounds and the resolved rectangular ancestor
clip must both contain the pointer. Future z-index/layer work must reorder paint
and hit-test data together.

Primary-button presses capture their target until release. A release over the
captured target emits `Clicked`; a release outside emits only `Released`.
Focusable controls receive focus on press, and losing window focus clears hover,
press, capture, and focus safely.

## Paint-only state changes

`Interaction` stores optional `QuadStyle` values for hovered, pressed, and
focused states. `QuadStyle` intentionally excludes layout and clipping. This
allows `LayoutEngine::repaint` to rebuild only ordered quad commands while
reusing Taffy geometry, shaped text, the glyph atlas, and hit regions.

`Button` is a composition helper in `argui-ui`: one interactive container, one
text child, layout style, paint styles, and no renderer-specific widget code.
Button labels default to no wrapping and their container has zero flex shrink,
so a wrapping row moves a complete button instead of truncating its label. Text
ink is clipped by explicit ancestor clips rather than its exact advance box,
which preserves glyph overhangs at the final character. The renderer still
receives only quads and text commands.

Keyboard traversal, arbitrary per-widget state, accessibility nodes, touch,
scrolling, text editing, IME, and rounded hit masks remain separate milestones.
