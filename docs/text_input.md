# Text input

`TextInput` is a composed UI element. WGPU still receives only ordinary quads
and glyphs; editing behavior does not leak into the renderer.

The retained state is keyed by the element's stable `NodeId`. It owns the UTF-8
value, caret, selection anchor, and temporary IME preedit. Cosmic Text supplies
the shaped caret stops, bidi-aware visual movement, selection rectangles, and
horizontal offset needed to keep a long single-line caret visible. Editing and
deletion always stop on grapheme boundaries.

Every caret position includes boundary affinity. At a Latin/RTL transition, the
same logical byte index may therefore retain two distinct visual positions.
Pointer hit testing, drag selection, arrows, and word navigation preserve that
affinity instead of snapping to an arbitrary side of the bidi run.

Single-line selection geometry follows the visual interval between the anchor
and focus stops. It therefore grows monotonically under a physical left-to-right
drag even when the Unicode logical order crosses several RTL runs.

Caret and selection changes reuse a bounded cache of shaped input buffers. They
update editing geometry and reposition prepared glyphs without rerunning Taffy
or rebuilding the complete text scene. Horizontal scrolling remains stable until
the caret actually leaves the visible interval.

Winit keyboard, modifier, and IME events are normalized by `argui-platform`.
`argui-runtime` routes them to the focused field and updates the native IME
candidate position. Tab and Shift-Tab traverse focusable elements. A changed
field emits `TextChanged`; Enter emits `Submitted`.

Copy and paste use `arboard` on native targets, including its Wayland data
control backend, and the browser Clipboard API on WASM. The browser operation is
asynchronous but returns through the same runtime event path. There is no DOM UI,
WebView, or second application implementation.

The current field is single-line and retains its initial value internally.
Controlled values, multiline editing, accessibility nodes, and richer IME
decoration are later extensions of the same state. The caret is visible and
movable now; blinking and interpolation belong to the shared animation scheduler
so an idle field never introduces a permanent redraw loop.
