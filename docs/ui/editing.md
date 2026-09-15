# Actions and text editing

## Scoped actions

`ActionScope` stores bindings created by `Context::on_action`. Resolution
starts at the focused node and walks ancestors until a modal boundary. A disabled
local binding blocks the same action in a parent.

```rust,ignore
const SAVE: ActionId = ActionId("mail.save-draft");
let state = ActionState::new("Save draft")
    .enabled(self.dirty)
    .shortcut(Shortcut::primary("s"));
let save = cx.on_action(SAVE, state.clone(), |model, _, cx| {
    model.save_in_memory();
    cx.notify();
});

Element::column([
    Button::new("save", &state.label, theme.button())
        .enabled(state.enabled)
        .build()
        .action(SAVE),
])
.action_scope(ActionScope::new([save])?)
```

Use the same `ActionState` for the binding and its controls. Invocation always
rechecks the retained scope. `prevent_default()` cancels a default action;
stopping propagation alone does not.

`ActionId::{COPY,CUT,PASTE,SELECT_ALL,UNDO,REDO}` use the editor pipeline and
can be overridden in a scope. Shortcuts are single key combinations and are
suppressed during IME composition.

Menus and command palettes carry an `ActionInvocation` plus current state.
Capture an invocation before moving focus when an overlay should act on an
editor. The application handles returned focus, close, toggle, and invoke
responses.

## Controlled editors

`TextInput` and `TextArea` are composed UI elements. Their retained state is
keyed by stable `NodeId` and stores UTF-8 text, caret, selection, IME preedit,
and scroll position.

Rebuilding with the same authored value preserves editing state. A different
external value replaces the buffer, cancels preedit, clears history, and clamps
positions to grapheme boundaries. Use a different element key when switching
documents.

Cosmic Text supplies shaping, bidi-aware caret stops, word boundaries, selection
rectangles, and wrapping. Editing never splits a grapheme. Boundary affinity
keeps the correct visual caret when one byte position has two positions at a
left-to-right/right-to-left transition.

Common behavior:

- double-click selects a word; triple-click selects a paragraph;
- Shift-click extends the existing anchor;
- arrows move visually and preserve bidi affinity;
- Home/End use the visual line; Ctrl/Cmd+Home/End use the document;
- word navigation and deletion follow platform modifiers;
- `TextInput` submits on Enter;
- `TextArea` inserts a newline on Enter and submits on Ctrl/Cmd+Enter.

Caret and selection changes reuse shaped input buffers. Taffy runs only when
layout inputs change. `TextArea` clips text, selection, and caret to its
viewport and shares one offset between caret reveal, wheel input, and scrollbar
dragging.

Winit keyboard and IME events are normalized by `argui-platform`.
`argui-runtime` routes them to the focused editor and updates the native IME
candidate position. Native clipboard uses `arboard`; Web uses the asynchronous
Clipboard API through the same runtime result path.

## Filters

`TextInputFilter` applies before a controlled value changes:

| Filter | Accepted draft |
| --- | --- |
| `Any` | arbitrary text |
| `Decimal` | decimal intermediate states such as empty, sign, or period |
| `Arithmetic` | arithmetic character set; expressions are not evaluated |

Application-specific ranges and meaning remain application validation.

## Undo and redo

History is per retained editor. Defaults are 100 transactions, 1 MiB of changed
UTF-8 text, and a 750 ms grouping interval. Configure
`HistoryConfig` on an editor or subtree; zero transactions disables history.

Contiguous typing and same-direction deletion group. Navigation, selection,
focus, paste, programmatic replacement, and IME commit end a group. Undo restores
selection direction and affinity. A new accepted edit after undo drops redo.
Rejected edits do not enter history.

`cx.edit_text(target, value)` performs one undoable replacement.
`can_undo`, `can_redo`, editor event snapshots, and scoped actions expose
availability.

## Password fields

`InputKind::Password` paints one bullet per grapheme and maps display positions
back to the real buffer. `TextPrivacy::RevealedPassword` reveals it visually
without enabling Copy, Cut, or undo history.

Semantic snapshots and DevTools summaries omit the value. The Web bridge remains
a real `input[type=password]` even while visually revealed. The owning Rust
model still contains the actual value; this API does not provide encrypted or
zeroized storage.

## Resize handles

Resize is normal composition. Attach a `PanGesture` to any element and handle
its typed start/change/end/cancel events in model state. The application owns
constraints, reset behavior, and persistence.

## Examples and tests

The Widget Gallery contains **Actions**, **Editing & Password**, and
**Async tasks** examples. The browser scenario
`crates/argui-widget-gallery/tests/pages/inputs.mjs` compares keyboard,
selection, pointer, and overlay behavior with HTML controls. Run it through the
[private display](../contributing/linux-testing.md).
