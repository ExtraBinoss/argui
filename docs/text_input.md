# Text input

`TextInput` is a composed UI element. WGPU still receives only ordinary quads
and glyphs; editing behavior does not leak into the renderer.

The retained state is keyed by the element's stable `NodeId`. It owns the UTF-8
value, caret, selection anchor, and temporary IME preedit. Cosmic Text supplies
the shaped caret stops, bidi-aware visual movement, selection rectangles, and
two-axis offset needed to keep a caret visible. Editing and
deletion always stop on grapheme boundaries.

Every caret position includes boundary affinity. At a Latin/RTL transition, the
same logical byte index may therefore retain two distinct visual positions.
Pointer hit testing, drag selection, arrows, and word navigation preserve that
affinity instead of snapping to an arbitrary side of the bidi run.

Selection geometry follows each visual line between the anchor and focus stops.
It therefore remains correct across soft wraps, explicit newlines, and mixed RTL
runs.

Caret and selection changes reuse a bounded cache of shaped input buffers. They
update editing geometry and reposition prepared glyphs without rerunning Taffy
or rebuilding the complete text scene. Editor scrolling remains stable until
the caret actually leaves the visible interval.

Winit keyboard, modifier, and IME events are normalized by `argui-platform`.
`argui-runtime` routes them to the focused field and updates the native IME
candidate position. Tab and Shift-Tab traverse focusable elements. A changed
field emits `TextChanged`. `TextInput` submits on Enter. `TextArea` inserts a
newline on Enter and submits on Command/Ctrl+Enter.

Copy and paste use `arboard` on native targets, including its Wayland data
control backend, and the browser Clipboard API on WASM. The browser operation is
asynchronous but returns through the same runtime event path. There is no DOM UI,
WebView, or second application implementation.

Both editors are controlled. Rebuilding with the same value preserves the exact
selection, IME preedit and scroll. A different external value replaces the
buffer, cancels preedit, and clamps caret/selection to grapheme boundaries.
`TextArea` uses Cosmic Text word-or-glyph wrapping and publishes a multiline
AccessKit node or a real browser `<textarea>`. Its retained viewport shapes the
complete wrapped content, clips text, selection and caret to the editor bounds,
and shares one offset between caret auto-reveal, wheel input and scrollbar
dragging. `scroll_config` controls axes, chaining and input speed;
`scrollbar` accepts any `ScrollbarStyle` without introducing renderer-specific
widget code. Scrollbar insets are independent on every side, so composite
widgets can reserve room for handles or controls. Pointer ownership follows
paint order: a later visual above the track receives input, while an exposed
track remains draggable.

`Resizable` is independent from text editing. It composes arbitrary content and
an arbitrary bottom-right handle, uses the shared pan gesture capture, exposes
horizontal/vertical/both axes, clamps to explicit min/max sizes, and reports
typed start/change/end/cancel phases. `argui-theme::WidgetAssets` provides the
standard GPU-rendered Tabler resize handle. Leaving a window only clears hover;
an active resize remains captured until pointer release. A real pointer cancel
or window focus loss still emits `Cancelled` and restores the starting size.
