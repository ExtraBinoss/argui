# Interaction in TSX and Rust

Argui sends platform input through the runtime to a retained UI tree. A native
node handles focus, hit testing, and event delivery; Solid and React callbacks
receive payloads from that same host. These are Argui events, not browser DOM
events or React synthetic events. `@argui/host` defines their public payloads
in [`events.ts`](../../packages/host/src/events.ts).

## Build a clickable surface

Use a widget `Button` for ordinary actions. For a custom control, keep the
focus and click boundary around the painted surface. `focusScope` is unpainted;
the child `rectangle` carries the hover, press, and focus paint. This Solid
example is a named keyboard and pointer action:

```tsx solid
import { useTheme } from '@argui/solid'
import type { WidgetTheme } from '@argui/widgets/solid'

export function SaveControl(props: { onSave: () => void }) {
  const theme = useTheme<WidgetTheme>()

  return <focusScope role="button" accessibleName="Save document"
    focusable keyboardActivation="enterOrSpace" onClick={props.onSave}>
    <rectangle width={120} height={36} radii={theme().radius}
      background={theme().secondary} hoverBackground={theme().secondaryHover}
      pressedBackground={theme().primary} focusBorderColor={theme().focusRing}>
      <row width="100%" height="100%" alignItems="center" justifyContent="center">
        <text color={theme().text}>Save</text>
      </row>
    </rectangle>
  </focusScope>
}
```

`keyboardActivation="enterOrSpace"` routes keyboard activation to `onClick`.
The same click path is available to assistive technology. `focusOnTabNavigation`
can remove a focusable custom node from sequential Tab navigation while
leaving it available for programmatic focus. Use `accessibleDisabled` or the
`enabled` control prop when the action is unavailable; do not only dim its
paint. See [Accessibility](accessibility.md) for state and action metadata.

## Direct input and state

Generated TSX declarations expose `onPointerDown`, `onPointerMove`,
`onPointerUp`, `onPointerCancel`, `onPointerEnter`, `onPointerLeave`, `onWheel`,
`onClick`, and `onContextMenu` on `touchArea`. `focusScope` exposes `onKey` and
`onCaptureKey` for keyboard input. Pointer payloads may include window and
local coordinates and node dimensions; read their optional fields before
doing geometry. A key payload includes `key`, `state`, modifiers, and `repeat`.
The `Slider` widget is the maintained example of pointer, keyboard, and
semantic value input sharing one state transition.

In Rust, `argui-ui` dispatches listeners attached with `Element::on` through
capture, target, and bubble phases. `Context::listener` creates model
callbacks. `prevent_default`, `stop_propagation`, and
`stop_immediate_propagation` control that shared dispatch; passive listeners
cannot prevent a default. Hit testing checks painted order in reverse and
respects ancestor clips. Rust `HitTestStyle` can select bounds, rounded or
elliptical hits and expand them with hit slop independently of the pixels.
There is no equivalent general hit-shape prop on every TSX primitive.

The native runtime keeps transitions beside retained node identities. In
Solid, use `<For>` for reactive keyed lists; in React, use stable `key` values.
Native `id` names a node for anchors, accessibility relations, and tests.
Changing a label should not change that ID. A list's framework `key` and
native `id` serve different purposes.

## Text editing

`InputField` wraps native `textInput`. It delegates caret, selection,
composition, undo, and text buffer handling to the native editor. Use
`value`/`onValueChange` for controlled text, `defaultValue` for native-owned
text, and `onSubmit` to receive the submitted string. With a controlled value,
apply `onValueChange` updates promptly so the displayed value can follow the
edit. A controlled field with no change callback must say `readOnly`.

```tsx solid
import { createSignal } from '@argui/solid'
import { InputField } from '@argui/widgets/solid'

export function SearchField(props: { search: (query: string) => void }) {
  const [query, setQuery] = createSignal('')
  return <InputField label="Search" type="search" value={query()}
    onValueChange={setQuery} onSubmit={props.search} />
}
```

At the lower Rust layer, `EventType::TextEdit` carries a range replacement
instead of copying the whole text for every keystroke. The native `Input`
event remains available when a complete edited value is needed. Details of
filters, IME, and history are in [Text editing](editing.md).

## Floating controls and focus

`Popover` owns its trigger anchor, `popupWindow`, dismissal, and focus
restoration. It is nonmodal by default: opening it leaves focus on the
trigger. Set `initialFocus="first"` to move focus to its first focusable
child. Outside pointer input or Escape dismisses it. `width` sizes the trigger
and `contentWidth` sizes the popup independently. Without `width`, the trigger
keeps its natural size. `accessibleLabel` overrides the visible trigger string
for its accessible name; `leading` and `trailing` accept application-owned
content. `closeLabel` can add a close button inside the popup. `placement`
chooses the preferred side and alignment before viewport fitting.

```tsx solid
import { createSignal } from '@argui/solid'
import { InputField, Popover } from '@argui/widgets/solid'

export function Filters() {
  const [open, setOpen] = createSignal(false)
  return <Popover trigger="Filters" open={open()} onOpenChange={setOpen}
    contentWidth={280} initialFocus="first">
    <InputField label="Query" type="search" defaultValue="" />
  </Popover>
}
```

Use `defaultOpen` for a locally managed initial state or pair `open` with
`onOpenChange` to control a Popover. `Select` uses a trapped option list,
supports arrows, Home, End, Enter, and Escape, and restores focus after
dismissal. `Tooltip` opens on hover or focus, closes after leaving the trigger
and popup or pressing Escape, and shows the plain string in `content`. Its
default `contentWidth` is 220 logical pixels. These
contracts are different from a modal dialog, which needs `containment="modal"`
and background focus exclusion at the native popup or scope level.

Popover and Tooltip use a translucent `overlaySurface` and blur by default.
Set `opaque` for the regular opaque surface, or `blur={false}` to keep the
translucency without backdrop blur. The `overlay*` theme tokens set their
padding, radius, border, blur, and in-window shadow. A requested native
outside-window popup omits that in-window shadow at its surface edge.

`allowOutsideWindow` on Popover, Select, Tooltip, or `popupWindow` requests a
separate native popup that can pass the owner window edge. Runtime support and
the window backend decide whether it is created; otherwise content falls back
inside the window. The current Linux native popup path uses X11. Do not use a
native popup demo as proof of Wayland, mobile, or Web support.
