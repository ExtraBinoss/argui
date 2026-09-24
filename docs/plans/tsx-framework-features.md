# Five functional priorities for TSX applications

This plan starts from `codex/dsl-gallery-live` after the optional `argui-media`
and native `GpuCanvas` work. Its goal is to let a Solid or React application use
Argui's native capabilities without rebuilding common controls or writing a
gallery-specific QuickJS bridge. The five milestones below are ordered for
delivery. Each milestone should produce a usable feature on both TSX adapters
before the next one begins.

The existing renderer, layout, text input, accessibility tree, virtual lists,
image/SVG support, i18n engine, popups, and native GPU canvas are foundations to
reuse. Do not replace them with a WebView or move pixels through JavaScript.

## 1. Application services from TSX

**Outcome:** a React or Solid component can call native application services
through one typed, asynchronous API, independent of the example gallery.

1. Extend the host protocol with request IDs, typed responses and errors,
   cancellation, and explicit window ownership. Keep UI tree commits separate
   from application requests. Pending requests must settle when their window or
   JS session closes.
2. Define a host-provided service registry so an application can supply its own
   native service without changing the renderer or QuickJS bootstrap. Ship
   built-in services for text clipboard, open/save file dialogs, application
   menus, and global shortcuts where the platform supports them. Expose
   capability queries and an explicit `unsupported` result elsewhere.
3. Keep large files and binary media native: return a file handle/path or an
   application-defined opaque ID, not the entire file as a JS array. Make
   permission and cancellation behavior visible to callers.
4. Expose the same TypeScript API to React and Solid. Demonstrate each service
   from a small TSX screen, with success, cancellation, and error states.

**Done when:** both adapters can request each service; responses go to the
correct live caller; closing or reloading a session does not deliver stale
results; platform differences are explicit. The bridge can later host storage
or network services without adding those concerns to the GUI engine.

## 2. Drag and drop across native windows and TSX

**Outcome:** users can drop files into an Argui view and drag application data
between controls or windows. This is the foundation for an editor timeline.

1. Add platform events for enter, move, leave, drop, and cancellation, with
   offered MIME types, allowed operation, pointer position, and source window.
   Add a bounded, lazy data-read path. A media-file drop should pass a native
   reference to its consumer instead of copying video bytes into QuickJS.
2. Route events through runtime hit testing and the retained tree. Define
   ownership and cleanup when the pointer leaves, the target unmounts, a
   window closes, or the source cancels.
3. Expose `onDragEnter`, `onDragOver`, `onDragLeave`, and `onDrop` plus a way to
   initiate an internal drag in both generated JSX contracts. Use one typed
   payload format in React and Solid; distinguish file, text, and app-defined
   data. Preserve keyboard and accessibility alternatives for reorder actions.
4. Connect desktop platform backends without hiding unsupported targets.

**Done when:** dropping one or more files into a TSX target, dragging an item
between two TSX targets, cancelling a drag, and dropping onto a removed target
all have defined behavior. A large file is never serialized through a UI
transaction.

## 3. Shared React and Solid widgets

**Outcome:** normal application screens need composition, not a new control
implementation for every app. Keep behavior and accessibility consistent while
allowing an application to provide its own palette and icons.

1. Build the first set on existing native primitives: `Checkbox`, `RadioGroup`,
   `Switch`, `Slider`, `Tabs`, `Dialog`, `Menu`/`MenuItem`, and `Progress`. Reuse
   the current `Button`, `InputField`, `Select`, and `Popover` contracts.
2. Make controlled state, disabled/busy/invalid states, keyboard interaction,
   focus restoration, semantic roles/actions, and touch targets part of each
   component's behavior. Provide both React and Solid entry points with the
   same public props where their framework models permit it.
3. Add a composable form layer for field labels, descriptions, validation
   errors, submit/reset, and grouped values. Do not embed a large form-state
   framework in the renderer. Add virtualized `TreeView` and `DataGrid` only
   after their interaction and accessibility contracts are defined; reuse the
   existing virtual-window engine.

**Done when:** a settings form, a modal confirmation, and a selectable menu can
be built from `@argui/widgets` in both frameworks without application-specific
focus or keyboard code. The core remains theme-neutral.

## 4. First-class custom native primitives and slots

**Outcome:** an app can expose a Rust-rendered or Rust-owned component to TSX
without editing Argui's built-in registry. This includes specialized editor
surfaces beyond the existing `GpuCanvas`.

1. Generate the JS contract and React/Solid JSX types from the application's
   actual schema registry, including custom primitives, their properties,
   events, slots, cardinality, and ABI hash. Built-in-only generation remains a
   convenience, not a limit on applications.
2. Extend host transactions to carry a slot ID for child insertion. Validate
   required/named slots and child counts before committing; keep the existing
   default `children` path for built-ins. Select a single JSX slot syntax that
   works in both React and Solid.
3. Complete the wire encoding/decoding of schema value types already advertised
   to TSX, including border, shadow, insets, and radii. Fail unsupported custom
   values at the JS boundary with a precise error.
4. Prove the path with one tiny application-defined Rust primitive used from
   React and Solid. It needs a property, event, and named slot; no video engine
   or separate rendering stack is required.

**Done when:** changing the application registry regenerates correct TSX types
and contract; a custom primitive mounts and updates on both adapters; missing
required slots and malformed values fail predictably rather than appearing to
work until a native commit.

## 5. Multiple application windows in TSX

**Outcome:** TSX owns the UI of more than one native window while native Rust
continues to own their rendering and lifecycle. Popovers remain separate from
application windows.

1. Expose typed create, close, focus, and window-state operations through the
   application-service bridge. Give each window a stable ID, configuration,
   event route, and independent native root. Scope responses and custom canvas
   registrations to the correct window where necessary.
2. Allow one React root per window and remove Solid's process-global active
   host assumption. Define how windows share application state without sharing
   reconciliation ownership or accidentally moving a native node between roots.
3. Route keyboard shortcuts, file dialogs, drag/drop, accessibility, scale
   changes, and close requests to the owning window. Closing a secondary
   window must release its TSX session, listeners, native tree, and GPU targets
   while leaving the rest of the app running.

**Done when:** a TSX main window can open a secondary window, edit shared app
state from either side, transfer an item between them, close and reopen the
secondary window, and keep focus and events bound to the right root.

## Delivery boundaries

- Implement one milestone at a time, with small native and TSX examples that
  exercise the delivered behavior. Keep React and Solid parity in each change.
- Preserve the renderer/runtime boundary: no React- or Solid-specific logic in
  layout, paint, or GPU code. Keep desktop Linux, Windows, and macOS behavior
  explicit; design the contracts for mobile and Web without claiming device
  support that has not been exercised.
- Run directly affected Rust and TypeScript tests during each milestone. For
  GUI checks on Linux, use the repository's private-display workflow. Run the
  repository's final quality gate once before each implementation commit, as
  required by `AGENTS.md`.
- Do not make built-in audio/video playback a prerequisite. A video editor can
  own GStreamer/GES and publish preview frames to `GpuCanvas`; milestones 1,
  2, 4, and 5 supply its application controls and interaction path.

## Next functional tranche

After these five milestones, expose writing direction in TSX so `@argui/i18n`
can drive RTL layout; add rich text spans and input intents such as password,
email, phone, and URL; preserve component state across successful hot reloads;
and broaden native accessibility actions and mobile interaction support.
These are feature work, not prerequisites for the first five milestones.
