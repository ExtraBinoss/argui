# ButtonGroup and Select: layout and motion pitfalls

## Center content inside the control's content box

Use a full-width `row` with `height="100%"`, `alignItems="center"` and
`justifyContent="center"` for an icon-only button. Give the SVG explicit equal
dimensions. On a pixel-rounded native layout, prefer even icon sizes (for
example 16px in a 24px button); an odd size can move the center by half a
pixel after rounding. A control's padding should reserve horizontal space independently
of its height: use `padding={{ start: 10, end: 10 }}` rather than `padding={10}`
on a 28 px text button. Border and padding both reduce the available content
box, just as with `box-sizing: border-box` on the web.

Text has a line box in addition to its font size. A 13 px label defaults to
a 16.25 px line box. If a marquee viewport and its moving rectangle are 20 px
high, bind `lineHeight={20}` to the text too, or center the natural text box in
a `row`. Setting only the text's `height` does not center its glyph baseline.
Keep trigger values in a centered row, with a bounded width for ellipsis.
Fix asymmetric whitespace in the SVG asset's viewBox rather than adding an
icon-library-specific pixel offset to every button.

## Animated segmented surfaces must share the cells' coordinates

Borders are stored as paint but `UiTree::resolved_layout_style` includes their
widths in layout. Absolute offsets therefore start inside the border.
For a 1 px border and four equal 4 px outer margins, use a row with 3 px
padding and an active surface with `inset={{ left: 3, top: 3 }}`. For `n`
equal cells in an outer `W × H` frame, cell width is `(W - 8) / n` and active
height is `H - 8`. Move one retained surface by `index * cellWidth`; use these
same dimensions for the focus scopes. Do not subtract padding on one axis
and forget the border on the other. Bound the icon/label stack so it fits
the resulting height, including its line height and gap. Fractional cell
widths may round by a pixel; compare the native layout bounds, not raw styles.

## A divider must follow the group's measured height

`ButtonGroupSeparator` once used a fixed 24 px height. A taller button or
input left a gap above and below the visible divider, even though the outer
group border covered the full control height. In a horizontal `row`, leave the
vertical divider's height unset and use `alignSelf="stretch"` with `width={1}`
and `shrink={0}`. The row can then size itself from its controls and stretch
the divider across that measured cross axis. Keep the group's rounded outer
clip; the separator belongs between the controls, inside the border.

Check mixed child heights, an explicitly tall group, an InputField beside a
Button, and RTL. A full-height divider should meet the inside of the top and
bottom group borders without protruding through them.

## A Select chevron needs one retained node

Select receives its trailing icon from the application; the widget must not
depend on a gallery icon package. Wrap that icon in one stable native
`container`. Bind its clockwise `rotation` to the same `expanded` value used
for the popup: 0 degrees when closed, 180 when open. Set `transitionMs` and a
timing curve on that container. Argui animates the property natively and
finishes at the target under reduced motion.

Replacing a down icon with an up icon makes the change abrupt. Driving angles
with a JavaScript timer rebuilds the tree on every frame. Rotating an arbitrary
unstable wrapper can also restart the transition on each update. Keep the
container's identity and layout size stable while the supplied icon turns.

Verify opening, choosing an option, Escape dismissal, and controlled `open`
state in both Solid and React. Check the actual desktop capture in addition to
the headless host's `rotation` and `transitionMs` properties.

Use native `hoverBackground` for pointer highlighting. Keep the authored active
index for keyboard navigation and reset it when the pointer enters or leaves
an option, so a departed pointer cannot leave a second row highlighted. A
selected option keeps its own selection paint. `focusScope` does not expose pointer enter/leave events. `touchArea` uses
`BoxOnly` and prevents its descendants from being pointer targets: wrapping
an entire option in it blocks a nested marquee label. Keep the native row
hover paint on the focus scope's rectangle and one reachable label target
for measurement/enter/leave. Reset keyboard activity from that label's events;
reset native motion when it leaves. Validate real hit regions, not only
callbacks delivered directly by a JavaScript test.

For a held native marquee, the outward leg occupies 35% of `loopMs`.
For approximately 14 logical pixels/second, use
`max(4000, ceil(overflow / 14 / 0.35 * 1000))`, with overflow measured from the
native intrinsic text width minus its bounded viewport. The native loop
accepts up to 86,400,000 ms. Do not cap long names at a short duration and make
them race across the viewport. Keep motion native and use `ScrollShadow`
for clipped edges; never estimate overflow by counting characters.

The gallery's fruit selector uses `variant="shadcn"`: align the selected row
with the trigger, including the native focus reveal scroll for long menus.
Only the default variant opens below. Without `contentWidth`, use native
`anchorWidth="matchAnchor"` with `anchorWidthOffset={2*S}` to follow actual
trigger bounds on resize. `placementCrossOffset={-S}` compensates the shadow
frame; the vertical offset must also include `S` and the panel border. Read
trigger width/height from click/key geometry, and never anchor a Select to the
pointer position. Keep the selected-row alignment while the pointer hovers
other options; keyboard activity changes should not move the popup.

## Returning to an identity transform can lose the compositor layer

A direct transform transition may animate smoothly toward a nonzero target and
then appear to snap back to zero. When a popup closes, its removal triggers a
new layout. The authored chevron transform is now identity, so deciding whether
to paint a compositor layer from that target alone drops the layer even while
the retained motion is still active. The engine can continue sampling values
every frame with no layer to present them; its final paint then reveals the
target in one jump. The same pattern affects a Switch thumb returning to its
start position. Layer opacity returning to 1 has the same failure mode.

At paint time, keep a compositor boundary while the retained node has an active
transform or layer-opacity transition, even if its authored target is identity.
Remove the boundary after the motion settles and repaints. Test a structural
change during the return transition, not just a simple property retarget. In a
private-display capture, sample both directions frame by frame; headless
property values alone cannot prove the compositor actually presents them.

A Switch can still *look* late on return if its track changes from light to
dark immediately while the dark thumb uses a slow-start easing curve. Animate
the track color with the thumb and use a curve that starts moving promptly in
both directions. A TSX `background` is a solid `Fill`, which used to switch at
the transition midpoint because the engine treated every `Fill` as discrete.
Interpolate colors when both endpoints are solid; keep changes between solid,
gradient, and absent fills discrete. Inspect pixel positions and colors across
the sequence before calling a slow-looking motion a skipped engine frame.
