# Retained responsive controls

Use these checks when clicks flash labels, a segmented indicator jumps, form
arrows disappear, or a native resize needs too much JavaScript work.

## Keep composition stable

A Solid JSX prop can be a getter that creates a component. Resolve a composed
header, trailing slot, or child once with `untrack(() => props.children)` when
the slot represents retained content. Internal signals and JSX effects remain
reactive. If the parent needs conditional content, pass a retained `Show` with
reactive branches; resolving a plain conditional once would freeze that choice.

Use `For` over stable primitive IDs and retrieve current option data through a
memoized lookup. Replacing metadata objects must not replace their controls.
Do not put a stable animated indicator inside a keyed `Show` whose object value
changes with dimensions: Solid may create its replacement before removing the
old native ID. Update the retained indicator's properties instead. Put changing
width and height on an unanimated sizing wrapper. For a fixed-width control, a
percentage-sized painted child can own the native transform transition. For a
control that stretches during native resizing, give its cells and indicator
relative widths. A native percentage-width offset can animate the active cell
while keeping the indicator width responsive; pixel offsets calculated from
the last JS size otherwise stay stale until the resize commit.

Separate initial loading, blocking project replacement, and queued saving.
Routine edits should not toggle every action's disabled state or substitute the
empty-preview message. Accept a new snapshot and its selection in one `batch`;
reset contextual tabs by selected identity rather than every metadata object.

## Fit actual native bounds

Native Select has a preferred theme width when `width` is absent. Inside a form
cell, set `width="100%"`, `minWidth={0}` and a suitable maximum width so the
whole trigger, including its fixed chevron, fits. A compact trigger and its menu
can have different widths: use `contentWidth` for readable menu headers. Native
Select reserves additional outside space for shadows; verify the painted menu
width separately from the popup's total width.

`containerScope` and `containerRules` belong to layout primitives such as
`container`, `row`, and `column`; a painted `rectangle` does not expose those
properties in the current schema. Query overrides accept the declared layout
styles, not arbitrary CSS such as `display`. Collapse a clipped label wrapper
to zero height to show only its icon, retaining the control's accessible name.
Collapse an optional control wrapper's width and height when a narrow toolbar
cannot fit it. Query thresholds evaluate actual native container bounds, so
they can adapt during engine-owned resizing without a JS resize callback.
Give the control itself `width="100%"` when it should grow with its panel:
`maxWidth="100%"` only caps a fixed preferred width, and cannot make it expand.
Collapse the icon/label gap along with a hidden label so the icon remains centered.

## Delay real hover hints

The native `tooltip` property is descriptive metadata for an application tooltip
host; setting it alone does not mount a visible popup. Reuse a shared hint
component anchored to a stable trigger ID. One timer on pointer entry can delay
opening; cancel it on exit, press, and disposal. Keep popup placement, opening
animation and dismissal native. A hint wrapper should not add a keyboard tab
stop or take focus from its control.

Pointer enter/leave are targeted native boundary events. Wrapping an existing
Button or Select in another TouchArea does not make the wrapper receive its
child's hover events. Place the hint trigger inside the existing focus scope,
around the icon/label's noninteractive painted content, so it is the actual
hit target and clicks still propagate to the control. Preserve the control's
pointer or disabled cursor on that trigger. A TouchArea also uses block layout;
place its painted content in a full-size `row` or `column` to preserve centering.
Check the SVG's center against the real button rectangle in both axes, not only
whether it fits within the button.

For hints that belong only to icon-only mode, keep the paint stable and put a
separate transparent hover region inside the same focus scope. Set that region
to zero height with clipping by default, then enable its full height with the
same native container query that hides the label. This removes it from hit
testing when the label is visible, without resize callbacks or moving the paint.
Check `UiTree.pointer_moved` and a real native press/release, rather than
delivering hover callbacks directly only.

Use an automatic content-sized popup with a bounded painted surface for hints.
Give its flex text wrapper `minWidth={0}` and its text `width="100%"` so long
content can wrap; omit a line clamp when the full hint must remain readable.
Measure both the popup and the surface: an auto-width detached block measured
against definite viewport space can still fill that space. The native portal
measurement must use intrinsic width, then apply viewport collision constraints.
Percentage portal sizes must keep their original viewport basis during that
second measure rather than applying the percentage to their already-sized bounds.

## Verify the mounted path

Build the actual bundle and mount it through the Rust host. Compare accepted
wire node identities before a click, while its service response is pending, and
after accepting the response. Check that unrelated labels and controls were not
removed. Measure fields and chevrons with `LayoutEngine` at minimum pane sizes,
and exercise native container queries on the same retained tree. Start a real
native resize, recompute layout while the pointer is still captured, cross each
breakpoint in both directions, and assert unchanged producer revision and no JS
move callbacks. Check actual text clips and hover hit regions in addition to a
zero-height label wrapper; a geometry-only check can miss overflowing paint.

For motion, reconcile the clicked state into `UiTree`, sample its initial frame,
then a time inside the transition and a time after its end. Native motion's
first sample establishes its start time; testing only that sample does not
demonstrate interpolation. For a delayed hint in Beam's current QuickJS timer
scheduler, initialize the timer with a tick before testing the delay boundary.
These checks prove retention, timing and geometry; frame-rate claims still need
presentation traces on the actual platform.
