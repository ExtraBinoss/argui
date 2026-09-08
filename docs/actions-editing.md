# Actions and retained editing

## Application actions

Actions are retained descriptions, not a global registry. An `ActionScope` holds
`ActionBinding`s created by `Context::on_action`. Its constructor rejects duplicate
IDs and conflicting shortcuts. Resolve from the focused node through its ancestors;
a modal focus scope is a boundary. A disabled local action blocks the same ID in a
parent. Hidden/disabled origins and removed captured nodes fail closed.

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
        .enabled(state.enabled).build().action(SAVE),
]).action_scope(ActionScope::new([save]).expect("unique actions"))
```

Use the same `ActionState` for bindings and controls. Controls remain controlled:
disabled styling is explicit, while invocation always rechecks the retained scope.
`UiTree::action_state` exposes live resolution to custom hosts. `Context::invoke_action`
queues an invocation on the UI thread. `ActionId::{COPY,CUT,PASTE,SELECT_ALL,UNDO,REDO}`
are built-ins using the editor/clipboard pipeline; applications can override them
in a scope. Existing Context selection commands use this same action resolution.

`Element::action_from(ActionInvocation::new(SAVE).at(origin))` captures a context
for menus/palettes. Capture before moving focus, using `UiEvent::focused_node()`
on PointerDown, or retain the last editor Focus event. Do not resolve by a reused
string key after a document has unmounted. Ordinary `.action(SAVE)` starts from
the activated control. UI `prevent_default()` cancels its action; stopping event
propagation alone does not cancel the default. Shortcuts use one key combination,
not multi-stroke chords, and are suppressed during IME composition. The web host's
browser-shortcut guard keeps reload shortcuts outside the canvas handlers.

Enable `argui/widget-menu` or `argui/widget-command-palette` independently (both
are included in `widgets-all`). `MenuItem` pairs an invocation with its current
state. `Menu::response` / `CommandPalette::response` expose Toggle, Close, Focus
and Invoke responses; handle them in the owner, call `cx.request_focus` or
`cx.invoke_action`, and notify after changing visibility/query. The widgets reuse
Button, Input and Popover, including outside dismissal, restored focus, blur and
keyboard navigation. There are no native OS menus or nested menu levels here.

Custom hosts dispatch event listeners first. A handler-less `UiEventKind::Action`
whose `should_dispatch()` succeeds is a deferred default: call `UiTree::invoke_action`
and process its resulting update. Do not send that event directly to an Entity
as though it were an ordinary listener. The bundled runtime handles this path.
AppModel wrappers must forward `take_ui_commands` alongside the other requests.

## Undo and redo

Each retained editor has its own history. The defaults are 100 transactions,
1 MiB of replacement text, and 750 ms between grouped keystrokes. Configure
`.text_history(HistoryConfig { .. })` on an editor or containing subtree, or
`UiTree::configure_text_history` in a custom host. Zero transactions disables it.
Changing the configuration clears existing transactions; reapplying it unchanged
preserves them.
The byte budget counts removed/inserted UTF-8 text, not allocator overhead.

Typing and same-direction deletes group while contiguous. Navigation, selection,
focus changes, paste, programmatic replacement and IME commits separate groups.
Preedit is not recorded. Undo/redo restore selection direction and caret affinity.
Undo followed by a new accepted edit drops redo. Rejected filtered edits do not.
An edit larger than the history budget still applies but clears the history.

Use `cx.edit_text(target, value)` for one undoable replacement. Updating the
authored value to a different value is an external reset: history and preedit are
cleared. Echoing the current editor value or rerendering the same authored value
preserves state. Give a newly selected document a different editor key.

`UiTree::can_undo/can_redo`, editor events' `edit_history()` snapshot, and the
shared actions expose availability. Standard shortcuts are primary+Z,
primary+Shift+Z, and Control+Y outside macOS. Explicit UI undo abandons preedit;
ordinary shortcuts do not interrupt an active composition.

Number accepts decimal intermediate states (including empty/sign/period).
Arithmetic filters its alphabet; it does not evaluate expressions or validate
application-specific numeric ranges.

## Password

`InputKind::Password` masks one bullet per grapheme, with display/real offset
mapping for caret and pointer selection. Use
`.text_privacy(TextPrivacy::RevealedPassword)` to reveal visually, controlled by
an accessible Button. Revealing never enables Copy, Cut or undo history.
Paste and Select all remain available. Native accessibility uses PasswordInput;
the browser bridge stays a real `input[type=password]`, even during visual reveal.

Semantic snapshots contain no password value. The browser control is synchronized
through a separate value path, not a semantic value/attribute. Element/UiTree debug
output, runtime event telemetry and DevTools summaries redact protected text.
Owner edit callbacks and the controlled Rust value still contain the actual text.
This is not a zeroizing allocator, encrypted storage or protection against code
running in the application process/browser origin. Never put a secret in its
label, description, key or an unrelated user-authored text element.

## Examples

Run `./scripts/serve-widget-gallery.sh` and open `/widgets/`:

- **Async tasks**: cancellable search, latest-result policy and parse errors.
- **Actions**: two local scopes, window fallback, modal isolation, menu/palette.
- **Editing & Password**: Unicode, filters, history, atomic edit, reset, reveal.
- **Draft Studio**: 256 offline drafts, cancellable search and virtualized results,
  retained editor history and shared save/reset commands. Save is in-memory only;
  reload discards changes. There is no mail backend or simulated disk write.
